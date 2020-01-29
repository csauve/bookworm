use log::*;
use crate::api::{ApiGameState, ApiDirection};
use crate::game::{Board, UnitAbs};
use crate::util::cartesian_product;

//these numbers are weak!!! time to optimize moar
const MAX_LOOKAHEAD_DEPTH: u8 = 3;
const PRIORITY_DIST: UnitAbs = 3;
const MIN_PRIORITY_SNAKES: UnitAbs = 1;

type Score = f32;

pub fn get_decision(game_state: &ApiGameState) -> ApiDirection {
    let root_turn_board = Board::from_api(game_state);
    let (score, path) = evaluate_board(&root_turn_board, MAX_LOOKAHEAD_DEPTH);
    debug!("Turn {} score {}: {:?}", game_state.turn, score, path);
    path.first().cloned().unwrap_or_else(|| root_turn_board.you().get_default_move())
}

fn heuristic(board: &Board) -> Score {
    let territories = board.get_territories();
    let indiv_scores = board.snakes.iter().enumerate().map(|(snake_index, snake)| {
        let territory = territories.get(snake_index).unwrap();
        //todo: tune heuristics (e.g. prevent from being too big)
        let h_control = territory.area as Score / board.area() as Score;
        let h_food = snake.health as Score / 100.0;
        let h_head_to_head = board.snakes.iter().enumerate()
            .filter(|(other_index, other)| *other_index == snake_index || snake.size() > other.size())
            .count() as Score /
            board.snakes.len() as Score;
        h_food * h_head_to_head * h_control
    }).collect::<Vec<_>>();

    let you_score = indiv_scores[0];
    let enemy_score_sum = indiv_scores.iter().skip(1).fold(0.0, |sum, s| sum + s);
    if indiv_scores.len() <= 2 {
        (you_score - enemy_score_sum) + 0.1
    } else {
        (you_score - enemy_score_sum / (indiv_scores.len() as Score - 1.0)) + 0.1
    }
}

//todo: use a deadline instead of max_depth?
//assumes the "you" index is 0 to avoid reshuffling complexity
fn evaluate_board(board: &Board, max_depth: u8) -> (Score, Vec<ApiDirection>) {
    if max_depth == 0 {
        return (heuristic(board), Vec::new());
    }

    //it's impractical to explore the entire turn tree, so filter moves out.
    //also ensure every snake makes at least their default move
    let moves_to_explore = board.get_free_snake_moves().iter().enumerate()
        .map(|(i, moves)| {
            let default_move = board.snakes[i].get_default_move();
            let is_priority_snake = i < MIN_PRIORITY_SNAKES ||
                (board.snakes[i].head() - board.you().head()).manhattan_dist() <= PRIORITY_DIST;
            if moves.is_empty() {
                vec![default_move]
            } else if is_priority_snake {
                moves.clone()
            } else if moves.contains(&default_move) {
                vec![default_move]
            } else {
                vec![moves[0]]
            }
        })
        .collect::<Vec<_>>();

    cartesian_product(&moves_to_explore).iter()
        .map(|moves| {
            let mut next_turn_board = board.clone();
            let you_move = moves[0];
            let dead_snake_indices = next_turn_board.advance(false, moves);
            if dead_snake_indices.iter().any(|(d, _)| *d == 0) {
                (0.0, vec![you_move])
            } else {
                let (score, mut path) = evaluate_board(&next_turn_board, max_depth - 1);
                path.insert(0, you_move);
                (score, path)
            }
        })
        .max_by(|(score_a, _), (score_b, _)| score_a.partial_cmp(score_b).unwrap())
        .unwrap_or_else(|| (0.0, vec![]))
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
