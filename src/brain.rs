use std::sync::Mutex;
use std::time::{SystemTime, Duration};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd, max};
use std::collections::BinaryHeap;
use log::*;
use log::Level::Debug;
use rayon::prelude::*;
use crate::api::{ApiDirection, ApiGameState, ALL_DIRS};
use crate::game::{Board, UnitAbs, FOOD_SPAWN_CHANCE};
use crate::util::{cartesian_product, draw_board};

//4 ^ 4 = 256
const MAX_PRIORITY_SNAKES: UnitAbs = 4;

type Score = f32;

struct FrontierBoard {
    board: Board,
    root_dir: Option<ApiDirection>,
    depth: usize,
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

//assume no NaN
#[inline]
fn min_f32(a: f32, b: f32) -> f32 {
    match a.partial_cmp(&b).unwrap() {
        Ordering::Less => a,
        _ => b
    }
}

//should be 1.0 if will win, 0.0 if will lose, in between otherwise
fn heuristic(board: &Board, snake_index: usize) -> Score {
    if snake_index == 0 && board.snakes.len() == 1 {
        return 1.0;
    }
    let territories = board.get_territories();
    let snake = board.snakes.get(snake_index).unwrap();
    let territory = territories.get(snake_index).unwrap();
    let total_area: UnitAbs = max(1, territories.iter().map(|terr| terr.area).sum());
    let h_control = territory.area as Score / total_area as Score;
    let h_food = {
        let turns_until_starve = snake.health;
        if turns_until_starve == 0 {
            0.0
        } else {
            territory.nearest_food.map(|nearest| {
                1.0 - min_f32(1.0, nearest as Score / turns_until_starve as Score)
            }).unwrap_or_else(|| {
                let p_food_spawn = FOOD_SPAWN_CHANCE as Score / 100.0;
                min_f32(1.0, p_food_spawn *
                    turns_until_starve as Score *
                    board.snakes.len() as Score /
                    total_area as Score
                )
            })
        }
    };
    let h_head_to_head = board.snakes.iter().enumerate()
        .filter(|(other_index, other)| {
            *other_index == snake_index || //dont need to worry about self
            other.size() < snake.size()  || //dont need to worry about small snakes
            (other.head() - snake.head()).manhattan_dist() > 2 //dont need to worry about distant snakes
        })
        .count() as Score / board.snakes.len() as Score;
    let h_snakes = 1.0 / board.snakes.len() as Score;

    h_food * h_head_to_head * h_control * h_snakes * h_snakes
}

//search the turn tree for a good and likely result, returning the first move to get there
pub fn get_decision(game_state: &ApiGameState, budget: Duration) -> ApiDirection {
    let start = SystemTime::now();
    let root_turn_board = Board::from_api(game_state);
    let mut n_considered: usize = 0;
    let mut decision = root_turn_board.you().get_default_move();

    let mut frontier: BinaryHeap<FrontierBoard> = BinaryHeap::new();
    frontier.push(FrontierBoard {
        board: root_turn_board,
        root_dir: None,
        depth: 0,
        h_score: 1.0, //dont bother with heuristic; we're gonna pop it first anyway
    });

    //live ur best life
    while let Some(leader) = frontier.pop() {
        if SystemTime::now().duration_since(start).unwrap() >= budget {
            if let Some(dir) = leader.root_dir {
                info!(
                    "Budget elapsed: n_considered={}, depth={}, score={}",
                    n_considered,
                    leader.depth,
                    leader.h_score
                );
                decision = dir;
                break;
            }
        }

        //figure out what possible moves each snake could make, including the `you` snake at index 0
        let mut snake_moves = leader.board.enumerate_snake_moves();

        //the cartesian product of snake moves can get large, so prune some away
        let you_head = leader.board.you().head();
        let closest_snakes = leader.board.get_closest_snakes_by_manhattan(you_head);
        for (snake_index, _dist) in closest_snakes.iter().skip(MAX_PRIORITY_SNAKES) {
            if let Some(dirs) = snake_moves.get_mut(*snake_index) {
                let default_move = leader.board.snakes.get(*snake_index).unwrap().get_default_move();
                if dirs.contains(&default_move) {
                    dirs.resize(1, default_move);
                } else {
                    dirs.truncate(1);
                }
            }
        }

        //fixed array indexed by ApiDirection; use insted of a HashMap to keep data on the stack
        let worst_outcomes: Mutex<[Option<FrontierBoard>; ALL_DIRS.len()]> = Mutex::new([None, None, None, None]);

        //YOU GET A CORE, YOU GET A CORE, YOU GET A CORE! EVERYBODY GETS A CORE!

        let move_space = cartesian_product(&snake_moves);
        n_considered += move_space.len();

        move_space.par_iter().for_each(|moves| {
            let mut next_board = leader.board.clone();
            let dead_snake_indices = next_board.advance(false, moves);
            let you_move = *moves.get(0).unwrap();
            let dir_index = you_move.as_index();

            //we are maintaining index 0 as "you"
            if dead_snake_indices.contains_key(&0) {
                worst_outcomes.lock().unwrap()[dir_index] = Some(FrontierBoard {
                    board: next_board,
                    root_dir: Some(leader.root_dir.unwrap_or(you_move)),
                    depth: leader.depth + 1,
                    h_score: std::f32::MIN,
                });
            } else {
                let next_h_score = heuristic(&next_board, 0);
                let is_new_worst = worst_outcomes.lock().unwrap()
                    .get(dir_index)
                    .unwrap()
                    .as_ref()
                    .map(|worst_outcome| next_h_score < worst_outcome.h_score)
                    .unwrap_or(true);

                if is_new_worst {
                    worst_outcomes.lock().unwrap()[dir_index] = Some(FrontierBoard {
                        board: next_board,
                        root_dir: Some(leader.root_dir.unwrap_or(you_move)),
                        depth: leader.depth + 1,
                        h_score: next_h_score * 0.25 + leader.h_score * 0.75,
                    });
                }
            };
        });

        //move the worst outcomes into the frontier so we can choose the best move, unless death is the worst case
        for worst_outcome in worst_outcomes.lock().unwrap().iter_mut() {
            if let Some(frontier_board) = worst_outcome.take() {
                if frontier_board.h_score != std::f32::MIN {
                    if log_enabled!(Debug) && frontier_board.depth == 1 {
                        debug!("Depth 1 option: dir={:?} score={}\n{}", frontier_board.root_dir, frontier_board.h_score, draw_board(&frontier_board.board));
                    }
                    frontier.push(frontier_board);
                }
            }
        }
    }

    if log_enabled!(Debug) {
        for frontier_board in frontier.iter().take(5) {
            debug!("Runner up: dir={:?} depth={} score={}\n{}", frontier_board.root_dir, frontier_board.depth, frontier_board.h_score, draw_board(&frontier_board.board));
        }
    }

    decision
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiDirection::*;
    use crate::api::*;
    use crate::util::init_logger;

    macro_rules! decide {
        ($s:expr) => {
            get_decision(&ApiGameState::parse_basic($s), Duration::from_millis(200))
        };
    }

    #[test]
    fn test_facing_wall() {
        let result = decide!("
        |  |  |  |
        |Y0|Y1|Y2|
        |  |  |  |
        ");
        assert_ne!(result, Left); //would hit wall
        assert_ne!(result, Right); //would hit self

        assert_eq!(Down, decide!("
        |Y0|Y1|Y2|
        |  |  |  |
        |  |  |  |
        "));
    }

    #[test]
    fn test_facing_self() {
        //could also go right to avoid tail, but would be trapped
        assert_eq!(Left, decide!("
        |  |  |  |  |  |
        |Y8|Y7|Y6|Y5|  |
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        "));
    }

    #[test]
    fn test_facing_other() {
        //should avoid being trapped between self and other snake
        assert_eq!(Left, decide!("
        |A3|  |  |  |  |
        |A2|A1|A0|Y5|Y6|
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        "));

        //todo: this is failing
        //will be trapped, but there is nowhere else to go
        assert_eq!(Right, decide!("
        |A2|  |Y6|  |
        |A1|A0|Y5|  |
        |Y0|  |Y4|  |
        |Y1|Y2|Y3|  |
        "));

        //don't really care which way it goes, just that it doesn't panic
        decide!("
        |  |  |A0|A1|
        |  |Y1|Y0|A2|
        |  |A5|A4|A3|
        |  |  |  |  |
        ");
    }

    #[test]
    fn test_lookahead_basic() {
        //looks like trapped, but actually next turn A's tail will move (assuming not stacked)
        assert_eq!(Right, decide!("
        |  |A1|A2|A3|
        |  |A0|Y0|A4|
        |  |  |Y1|  |
        "));
    }

    #[test]
    fn test_lookahead_avoid_dead_end() {
        //going Up has more space now but is a dead end, while B's tail will move and open up space
        assert_eq!(Right, decide!("
        |B0|  |  |  |  |  |  |
        |B1|B2|B3|B4|B5|  |  |
        |  |  |  |  |B6|B7|  |
        |A3|A2|A1|Y0|  |B8|  |
        |A4|A5|A0|Y1|C4|C3|  |
        |A7|A6|  |Y2|  |C2|  |
        |A8|A9|  |Y3|C0|C1|  |
        "));
    }

    //todo: work on this in heuristics
    // #[test]
    fn test_lookahead_best_dead_end() {
        //both options are a dead end, but going Up has more turns left (hope a snake dies and frees us)
        assert_eq!(Up, decide!("
        |B0 |   |   |   |   |   |   |
        |B1 |B2 |B3 |B4 |B5 |   |   |
        |   |   |   |   |B6 |B7 |   |
        |A3 |A2 |A1 |Y0 |   |B8 |   |
        |A4 |A5 |A0 |Y1 |B10|B9 |   |
        |A7 |A6 |   |Y2 |B11|   |   |
        |A8 |A9 |   |Y3 |B12|   |   |
        "));
    }

    #[test]
    fn test_almost_trapped() {
        assert_eq!(Up, decide!("
        |  |  |  |  |  |
        |  |A0|  |  |  |
        |  |A1|  |  |  |
        |Y0|A2|  |  |  |
        |Y1|A3|A4|  |  |
        |Y2|Y3|Y4|  |  |
        "));
    }

    #[test]
    fn test_trap_enemy() {
        //we have the opportunity to trap the enemy snake and keep
        assert_ne!(Right, decide!("
        |  |  |  |  |  |
        |  |Y0|  |  |  |
        |  |Y1|  |  |  |
        |A0|Y2|  |  |  |
        |A1|Y3|Y4|  |  |
        |A2|A3|A4|  |  |
        "));
    }

    #[test]
    fn test_enemy_already_trapped() {
        //enemy is already trapped; don't get trapped ourselves
        assert_eq!(Right, decide!("
        |  |Y0|  |  |  |
        |  |Y1|  |  |  |
        |A0|Y2|  |  |  |
        |A1|Y3|Y4|  |  |
        |A2|A3|A4|  |  |
        "));
    }

    //todo: the snake seems to be finding benefit in delaying this kill?
    // #[test]
    fn test_head_to_head_kill() {
        init_logger();
        //we have the opportunity to kill this enemy in a head-to-head collision
        assert_eq!(Up, decide!("
        |  |  |A2|  |  |
        |  |C0|A1|B0|  |
        |C2|C1|A0|B1|B2|
        |C3|  |  |  |B3|
        |  |  |Y0|  |  |
        |  |  |Y1|  |  |
        |  |  |Y2|  |  |
        |  |  |Y3|  |  |
        "));
    }

    //turn 36: https://play.battlesnake.com/g/a70e0095-5534-421c-9c0d-b464466ac554/
    #[test]
    fn test_avoid_trap_opportunity() {
        //if we go up, we will either die in head-to-head or give B the opportunity to trap us
        assert_ne!(Up, decide!("
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
        "));
    }

    //turn 20: https://play.battlesnake.com/g/f918c780-ef11-45ca-bd54-a2b9fb2dfc1e/
    #[test]
    fn test_avoid_head_to_head_death() {
        assert_eq!(Up, decide!("
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
        "));
    }

    //turn 3: https://play.battlesnake.com/g/343b8980-534d-47ec-936b-cfa241dc7531/
    #[test]
    fn test_avoid_head_to_head_death2() {
        assert_ne!(Down, decide!("
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |Y2|  |  |  |  |  |  |  |  |  |
        |  |Y1|  |  |  |  |  |  |  |  |  |
        |  |Y0|  |()|  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |()|  |  |  |
        |  |A0|  |  |  |  |  |  |  |  |  |
        |()|A1|  |  |  |  |  |  |  |  |  |
        |  |A2|  |  |B1|B2|  |  |  |  |  |
        |  |  |  |  |B0|  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        +Y97
        +A97
        +B97
        "));
    }

    //turn 99: https://play.battlesnake.com/g/4d5b00be-6036-4dc7-b0a3-78bb20d1451f/
    #[test]
    fn test_avoid_starvation() {
        init_logger();
        assert_eq!(Right, decide!("
        |  |  |  |()|  |  |  |  |  |A2|A1|
        |  |  |()|  |  |  |  |  |  |A3|A0|
        |  |  |()|  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |()|  |  |  |
        |  |  |  |  |B2|B1|B0|  |  |  |  |
        |  |  |  |  |B3|()|  |()|  |  |  |
        |  |  |  |  |B4|  |  |  |  |  |  |
        |  |  |  |()|B5|  |  |  |()|  |  |
        |Y2|  |  |  |B6|B7|B8|  |  |  |()|
        |Y1|Y0|()|  |  |  |  |  |  |  |  |
        +Y1
        +A90
        +B95
        "));
    }
}
