use std::sync::Mutex;
use std::time::{SystemTime, Duration};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BinaryHeap;
use log::*;
use rayon::prelude::*;
use crate::api::{ApiDirection, ApiGameState, ALL_DIRS};
use crate::game::{Board, UnitAbs};
use crate::util::cartesian_product;

const IGNORE_MOVES_DIST: UnitAbs = 5;

type Score = f32;

struct FrontierBoard {
    board: Board,
    root_dir: Option<ApiDirection>,
    h_score: Score,
}

impl Ord for FrontierBoard {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap() //assume no NaN scores
    }
}

impl PartialOrd for FrontierBoard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.h_score.partial_cmp(&other.h_score)
    }
}

impl Eq for FrontierBoard {}

impl PartialEq for FrontierBoard {
    fn eq(&self, other: &Self) -> bool {
        self.h_score == other.h_score
    }
}

//should be 1.0 if will win, 0.0 if will lose, in between otherwise
fn heuristic(board: &Board, snake_index: usize) -> Score {
    let territories = board.get_territories();
    let snake = board.snakes.get(snake_index).unwrap();
    let territory = territories.get(snake_index).unwrap();
    let h_control = territory.area as Score / board.area() as Score;
    let h_food = {
        let food_density = if territory.area == 0 {
            0.0
        } else {
            territory.num_food as Score / territory.area as Score
        };
        let turns_until_starve = snake.health;
        if territory.nearest_food.map(|nearest| nearest <= turns_until_starve as usize).unwrap_or(false) {
            1.0
        } else {
            let expected_food = turns_until_starve as Score * food_density;
            match expected_food.partial_cmp(&1.0).unwrap() {
                Ordering::Less => expected_food,
                _ => 1.0,
            }
        }
    };
    let h_head_to_head = board.snakes.iter().enumerate()
        .filter(|(other_index, other)| {
            *other_index == snake_index || //dont need to worry about self
            snake.size() > other.size() || //dont need to worry about small snakes
            (snake.head() - other.head()).manhattan_dist() > 2 //dont need to worry about distant snakes
        })
        .count() as Score / board.snakes.len() as Score;
    h_food * h_head_to_head * h_control
}

//search the turn tree for a good and likely result, returning the first move to get there
pub fn get_decision(game_state: &ApiGameState, budget: Duration) -> ApiDirection {
    let start = SystemTime::now();

    let root_turn_board = Board::from_api(game_state);
    let default_move = root_turn_board.you().get_default_move();

    let mut frontier: BinaryHeap<FrontierBoard> = BinaryHeap::new();
    frontier.push(FrontierBoard {
        board: root_turn_board,
        root_dir: None,
        h_score: 1.0, //dont bother with heuristic; we're gonna pop it first anyway
    });

    //live ur best life
    while let Some(leader) = frontier.pop() {
        if SystemTime::now().duration_since(start).unwrap() >= budget {
            return leader.root_dir.unwrap_or(default_move);
        }

        //fixed array indexed by ApiDirection; use insted of a HashMap to keep data on the stack
        let worst_outcomes: Mutex<[Option<FrontierBoard>; ALL_DIRS.len()]> = Mutex::new([None, None, None, None]);

        //todo: this can get pretty extreme for an 8 player game -- prune some moves which don't matter much?
        let mut snake_moves = leader.board.enumerate_snake_moves();
        let you_head = leader.board.you().head();
        for (other_index, dirs) in snake_moves.iter_mut().enumerate().skip(1) {
            let other = leader.board.snakes.get(other_index).unwrap();
            if (other.head() - you_head).manhattan_dist() > IGNORE_MOVES_DIST {
                let default_move = other.get_default_move();
                if dirs.contains(&default_move) {
                    dirs.resize(1, default_move);
                } else {
                    dirs.truncate(1);
                }
            }
        }

        //YOU GET A CORE, YOU GET A CORE, YOU GET A CORE! EVERYBODY GETS A CORE!
        cartesian_product(&snake_moves).par_iter().for_each(|moves| {
            let mut next_board = leader.board.clone();
            let dead_snake_indices = next_board.advance(false, moves);

            //we are maintaining index 0 as "you"
            let dir_index = moves.get(0).unwrap().as_index();
            let next_h_score = if dead_snake_indices.contains_key(&0) {
                std::f32::MIN
            } else {
                heuristic(&next_board, 0)
            };

            let is_new_worst = worst_outcomes.lock().unwrap()
                .get(dir_index)
                .unwrap()
                .as_ref()
                .map(|worst_outcome| next_h_score < worst_outcome.h_score)
                .unwrap_or(true);

            if is_new_worst {
                worst_outcomes.lock().unwrap()[dir_index] = Some(FrontierBoard {
                    board: next_board,
                    root_dir: Some(leader.root_dir.unwrap_or_else(|| *moves.get(0).unwrap())),
                    h_score: next_h_score * 0.25 + leader.h_score * 0.75,
                });
            }
        });

        //move the worst outcomes into the frontier
        for worst_outcome in worst_outcomes.lock().unwrap().iter_mut() {
            if let Some(frontier_board) = worst_outcome.take() {
                frontier.push(frontier_board);
            }
        }
    }

    //game over, man
    default_move
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiDirection::*;
    use crate::api::*;

    macro_rules! decide {
        ($s:expr) => {
            get_decision(&ApiGameState::parse_basic($s), Duration::from_millis(250))
        };
    }

    #[test]
    fn test_facing_wall() {
        let result = decide!(
            "
        |  |  |  |
        |Y0|Y1|Y2|
        |  |  |  |
        "
        );
        assert_ne!(result, Left); //would hit wall
        assert_ne!(result, Right); //would hit self

        assert_eq!(
            Down,
            decide!(
                "
        |Y0|Y1|Y2|
        |  |  |  |
        |  |  |  |
        "
            )
        );
    }

    #[test]
    fn test_facing_self() {
        //could also go right to avoid tail, but would be trapped
        assert_eq!(
            Left,
            decide!(
                "
        |  |  |  |  |  |
        |Y8|Y7|Y6|Y5|  |
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        "
            )
        );
    }

    #[test]
    fn test_facing_other() {
        //should avoid being trapped between self and other snake
        assert_eq!(
            Left,
            decide!(
                "
        |A3|  |  |  |  |
        |A2|A1|A0|Y5|Y6|
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        "
            )
        );

        //todo: this is failing
        //will be trapped, but there is nowhere else to go
        assert_eq!(
            Right,
            decide!(
                "
        |A2|  |Y6|  |
        |A1|A0|Y5|  |
        |Y0|  |Y4|  |
        |Y1|Y2|Y3|  |
        "
            )
        );

        //don't really care which way it goes, just that it doesn't panic
        decide!(
            "
        |  |  |A0|A1|
        |  |Y1|Y0|A2|
        |  |A5|A4|A3|
        |  |  |  |  |
        "
        );
    }

    #[test]
    fn test_lookahead_basic() {
        //looks like trapped, but actually next turn A's tail will move (assuming not stacked)
        assert_eq!(
            Right,
            decide!(
                "
        |  |A1|A2|A3|
        |  |A0|Y0|A4|
        |  |  |Y1|  |
        "
            )
        );
    }

    #[test]
    fn test_lookahead_avoid_dead_end() {
        //going Up has more space now but is a dead end, while B's tail will move and open up space
        assert_eq!(
            Right,
            decide!(
                "
        |B0|  |  |  |  |  |  |
        |B1|B2|B3|B4|B5|  |  |
        |  |  |  |  |B6|B7|  |
        |A3|A2|A1|Y0|  |B8|  |
        |A4|A5|A0|Y1|C4|C3|  |
        |A7|A6|  |Y2|  |C2|  |
        |A8|A9|  |Y3|C0|C1|  |
        "
            )
        );
    }

    //todo: work on this in heuristics
    // #[test]
    fn test_lookahead_best_dead_end() {
        //both options are a dead end, but going Up has more turns left (hope a snake dies and frees us)
        assert_eq!(
            Up,
            decide!(
                "
        |B0 |   |   |   |   |   |   |
        |B1 |B2 |B3 |B4 |B5 |   |   |
        |   |   |   |   |B6 |B7 |   |
        |A3 |A2 |A1 |Y0 |   |B8 |   |
        |A4 |A5 |A0 |Y1 |B10|B9 |   |
        |A7 |A6 |   |Y2 |B11|   |   |
        |A8 |A9 |   |Y3 |B12|   |   |
        "
            )
        );
    }

    #[test]
    fn test_almost_trapped() {
        assert_eq!(
            Up,
            decide!(
                "
        |  |  |  |  |  |
        |  |A0|  |  |  |
        |  |A1|  |  |  |
        |Y0|A2|  |  |  |
        |Y1|A3|A4|  |  |
        |Y2|Y3|Y4|  |  |
        "
            )
        );
    }

    #[test]
    fn test_trap_enemy() {
        //we have the opportunity to trap the enemy snake and keep
        assert_ne!(
            Right,
            decide!(
                "
        |  |  |  |  |  |
        |  |Y0|  |  |  |
        |  |Y1|  |  |  |
        |A0|Y2|  |  |  |
        |A1|Y3|Y4|  |  |
        |A2|A3|A4|  |  |
        "
            )
        );
    }

    #[test]
    fn test_enemy_already_trapped() {
        //enemy is already trapped; don't get trapped ourselves
        assert_eq!(
            Right,
            decide!(
                "
        |  |Y0|  |  |  |
        |  |Y1|  |  |  |
        |A0|Y2|  |  |  |
        |A1|Y3|Y4|  |  |
        |A2|A3|A4|  |  |
        "
            )
        );
    }

    #[test]
    fn test_head_to_head_kill() {
        //we have the opportunity to kill this enemy in a head-to-head collision
        assert_eq!(
            Up,
            decide!(
                "
        |  |C0|A1|B0|  |
        |C2|C1|A0|B1|B2|
        |C3|  |  |  |B3|
        |  |  |Y0|  |  |
        |  |  |Y1|  |  |
        |  |  |Y2|  |  |
        "
            )
        );
    }

    //turn 36: https://play.battlesnake.com/g/a70e0095-5534-421c-9c0d-b464466ac554/
    #[test]
    fn test_avoid_trap_opportunity() {
        //if we go up, we will either die in head-to-head or give B the opportunity to trap us
        assert_ne!(
            Up,
            decide!(
                "
        |  |  |  |  |  |()|A0|A1|A2|A3|  |
        |B5|B4|B3|B2|Y1|Y0|  |  |  |  |  |
        |B6|  |B0|B1|Y2|  |  |  |  |  |  |
        |  |  |  |  |Y3|  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |C2|C1|C0|  |  |  |  |  |  |  |  |
        |C3|  |  |  |  |  |  |  |  |  |  |
        |C4|  |  |  |  |  |  |  |  |  |  |
        |C5|  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        +Y66
        +A72
        +B95
        +C71
        "
            )
        );
    }

    //turn 20: https://play.battlesnake.com/g/f918c780-ef11-45ca-bd54-a2b9fb2dfc1e/
    #[test]
    fn test_avoid_head_to_head_death() {
        assert_eq!(
            Up,
            decide!(
                "
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |()|  |  |  |  |  |  |  |  |  |
        |C0|C1|C2|  |  |()|  |  |  |  |  |
        |  |  |C3|C4|  |  |  |  |  |  |  |
        |  |D3|D2|D1|  |Y3|Y2|Y1|Y0|  |  |
        |  |  |  |D0|  |  |  |  |  |A0|  |
        |  |  |  |  |  |  |B2|B1|  |A1|  |
        |  |  |  |  |  |  |  |B0|  |A2|  |
        |  |  |  |  |  |  |  |  |  |A3|  |
        |  |  |  |  |  |  |  |  |  |A4|A5|
        |  |  |  |  |  |  |  |  |  |  |  |
        +Y84
        +A95
        +B80
        +C93
        +D98
        "
            )
        );
    }
}
