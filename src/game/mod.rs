pub mod coord;
pub mod offset;
pub mod path;
pub mod snake;
pub mod turn;
mod util;

use std::collections::HashMap;
use crate::api::{ApiGameState, ApiDirection};
use coord::{Coord, Unit};
use turn::{Turn, AdvanceResult};
use util::cartesian_product;

const MAX_LOOKAHEAD_DEPTH: u8 = 3;

type Score = f32;

pub struct Game {
    bound: Coord,
    root_turn: Turn,
}

impl Game {
    pub fn init(game_state: &ApiGameState) -> Game {
        Game {
            bound: Coord::new(
                game_state.board.width as Unit - 1,
                game_state.board.height as Unit - 1
            ),
            root_turn: Turn::init(game_state),
        }
    }

    pub fn update(&mut self, game_state: &ApiGameState) {
        //don't really expect this to change, but just in case!
        self.bound = Coord::new(
            game_state.board.width as Unit - 1,
            game_state.board.height as Unit - 1
        );
        self.root_turn.update(game_state);
    }

    pub fn get_decision(&self) -> ApiDirection {
        let (dir, _score) = Game::evaluate_turn(&self.root_turn, self.bound, MAX_LOOKAHEAD_DEPTH);
        dir
    }

    //The space of next turns is basically the cartesian product of the sets of each snake's
    //possible moves. With a non-trivial number of snakes, the turn tree gets very big very
    //quickly. We will need to limit our calculation "budget" to certain subtrees; assume neither
    //we nor enemy snakes will choose to die when other options exist, and ignore snakes that
    //are not likely to affect us in this turn. The value/confidence of turn prediction also
    //decreases with depth because we cannot know where food will spawn, when enemies will do
    //something buggy like crash into a wall, or the likelihoods of the moves enemies will make.
    //We should instead guide longer-term planning and strategy with heuristics, and consider
    //the turn tree to be like "guard rails" that prevent poor decisions in the short term like
    //local maxima in the heuristic.
    fn evaluate_turn(turn: &Turn, bound: Coord, max_depth: u8) -> (ApiDirection, Score) {
        if max_depth == 0 {
            //todo: use a heuristic
            return (ApiDirection::Up, 1.0);
        }

        //todo: reduce number of moves to help prune turn tree
        let legal_moves = turn.get_legal_moves(bound);
        let mut by_you_move: HashMap<ApiDirection, Vec<Score>> = HashMap::new();

        for moves in cartesian_product(&legal_moves).iter() {
            let mut future_turn = turn.clone();
            let you_move = moves[0];
            let score = match future_turn.advance(moves, bound) {
                AdvanceResult::YouLive => {
                    //todo: decide to use a heuristic instead, given some condition
                    let (_, score) = Game::evaluate_turn(&future_turn, bound, max_depth - 1);
                    score
                },
                AdvanceResult::YouDie => {
                    0.0
                },
            };
            if let Some(values) = by_you_move.get_mut(&you_move) {
                values.push(score);
            } else {
                by_you_move.insert(you_move.clone(), vec![score]);
            }
        }

        let average_scores = by_you_move.iter()
            .map(|(dir, scores)| (*dir, scores.iter().sum::<Score>() / scores.len() as f32))
            .collect::<Vec<_>>();

        if max_depth == MAX_LOOKAHEAD_DEPTH {
            dbg!(&average_scores);
        }

        average_scores.iter().cloned()
            .max_by(|(_, score_a), (_, score_b)| score_a.partial_cmp(score_b).unwrap())
            //if we can't find a move, just pick default and pray to snake jesusSsSSss
            .unwrap_or_else(|| (turn.you().get_default_move(), 1.0))
    }

    pub fn width(&self) -> Unit {
        self.bound.x + 1
    }

    pub fn height(&self) -> Unit {
        self.bound.y + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::*;
    use crate::api::ApiDirection::*;

    macro_rules! decide {
        ($s:expr) => (Game::init(&ApiGameState::parse_basic($s)).get_decision());
    }

    #[test]
    fn test_init() {
        let api_game = ApiGameState::parse_basic("
        |  |()|  |
        |  |  |Y0|
        |A0|A1|Y1|
        |  |A2|  |
        |  |  |  |
        ");

        let game = Game::init(&api_game);

        assert_eq!(game.height(), 5);
        assert_eq!(game.width(), 3);

        let turn = &game.root_turn;
        assert_eq!(turn.food, vec![Coord::new(1, 0)]);
        assert_eq!(turn.enemies()[0].head(), Coord::new(0, 2));
        assert_eq!(turn.enemies()[0].tail(), Coord::new(1, 3));
        assert_eq!(turn.you().head(), Coord::new(2, 1));
        assert_eq!(turn.you().tail(), Coord::new(2, 2));
    }

    #[test]
    fn test_facing_wall() {
        let result = decide!("
        |  |  |  |
        |Y0|Y1|  |
        |  |  |  |
        ");
        assert_ne!(result, Left); //would hit wall
        assert_ne!(result, Right); //would hit self

        assert_eq!(Down, decide!("
        |Y0|Y1|  |
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

    #[test]
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
}
