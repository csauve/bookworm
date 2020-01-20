pub mod coord;
pub mod offset;
pub mod path;
pub mod snake;
pub mod turn;
mod util;

use std::collections::HashMap;
// use std::sync::Mutex;
// use rayon::prelude::*;
use log::*;
use crate::api::{ApiGameState, ApiDirection};
use turn::{Turn, AdvanceResult, Territory};
use std::cmp::{max, min};
use util::cartesian_product;

const MAX_LOOKAHEAD_DEPTH: u8 = 1;

type Score = f32;

pub fn get_decision(game_state: &ApiGameState) -> ApiDirection {
    let root_turn = Turn::init(game_state);

    let (dir, _score) = evaluate_turn(&root_turn, MAX_LOOKAHEAD_DEPTH);
    dir
}

//should return a value 0-1 representing surviveability of the snake
fn heuristic(snake_index: usize, turn: &Turn, territories: &[Territory]) -> Option<Score> {
    turn.snakes.get(snake_index).map(|snake| {
        let territory = territories.get(snake_index).unwrap();
        //todo: tune heuristics (e.g. prevent from being too big)
        let h_control = territory.area as Score / turn.area() as Score;
        let h_food = snake.health as Score / 100.0;
        let h_head_to_head = turn.snakes.iter().enumerate()
            .filter(|(other_index, other)| *other_index == snake_index || snake.size() > other.size())
            .count() as Score /
            turn.snakes.len() as Score;
        h_food * h_head_to_head * h_control
    })
}

fn evaluate_turn(turn: &Turn, max_depth: u8) -> (ApiDirection, Score) {
    debug!("Evaluating turn at max_depth {}", max_depth);

    let territories = turn.get_territories();

    //todo: are there other conditions we can bail out on?
    if max_depth == 0 {
        return (ApiDirection::Up, heuristic(0, turn, &territories).unwrap());
    }

    //todo: add mid-term planning with "plays"; use `territories`

    //todo: prune enemy moves which don't affect us; and by max enemy scores
    let legal_moves = turn.get_free_snake_moves();
    let mut by_you_move: HashMap<ApiDirection, Vec<Score>> = HashMap::new();

    for moves in cartesian_product(&legal_moves).iter() {
        let mut future_turn = turn.clone();
        let you_move = moves[0];
        let score = match future_turn.advance(moves) {
            AdvanceResult::YouLive => {
                let (_, score) = evaluate_turn(&future_turn, max_depth - 1);
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
        .map(|(dir, scores)| (*dir, scores.iter().sum::<Score>() / scores.len() as Score))
        .collect::<Vec<_>>();

    if max_depth == MAX_LOOKAHEAD_DEPTH {
        dbg!(&average_scores);
    }

    average_scores.iter().cloned()
        .max_by(|(_, score_a), (_, score_b)| score_a.partial_cmp(score_b).unwrap())
        //if we can't find a move, just pick default and pray to snake jesusSsSSss
        .unwrap_or_else(|| (turn.you().get_default_move(), 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::*;
    use crate::api::ApiDirection::*;

    macro_rules! decide {
        ($s:expr) => (get_decision(&ApiGameState::parse_basic($s)));
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

    #[test]
    fn test_uncertain_future() {
        //The A snake could easily trap us, so there should be some uncertainty in the score
        //todo: check score range
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
        assert_eq!(Left, decide!("
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

    #[test]
    fn test_head_to_head_kill() {
        //we have the opportunity to kill this enemy in a head-to-head collision
        assert_eq!(Up, decide!("
        |  |C0|A1|B0|  |
        |C2|C1|A0|B1|B2|
        |C3|  |  |  |B3|
        |  |  |Y0|  |  |
        |  |  |Y1|  |  |
        |  |  |Y2|  |  |
        "));
    }
}
