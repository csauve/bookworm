mod coord;
mod offset;
mod path;
mod snake;
mod turn;

use crate::api::{ApiGameState, ApiDirection};
use coord::*;

pub struct Game {
    bound: Coord,
    current_turn: Turn,
}

impl Game {
    pub fn init(game_state: &ApiGameState) -> Game {
        Game {
            bound: Coord::new(
                game_state.board.width as Unit - 1,
                game_state.board.height as Unit - 1
            ),
            current_turn: Turn::init(game_state),
        }
    }

    pub fn update(&mut self, game_state: &ApiGameState) {
        //don't really expect this to change, but just in case!
        self.bound = Coord::new(
            game_state.board.width as Unit - 1,
            game_state.board.height as Unit - 1
        );
        self.current_turn.update(game_state);
    }

    pub fn width(&self) -> Unit {
        self.bound.x + 1
    }

    pub fn height(&self) -> Unit {
        self.bound.y + 1
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
    pub fn get_decision(&self) -> Option<ApiDirection> {
        //todo: can state be broken up in a way that allows memoization, avoiding cycles?
        //todo: store probabilities and scores in the structure; update when invalidated?

        //todo
        Some(ApiDirection::Up)
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

        let turn = &game.current_turn;
        assert_eq!(turn.food, vec![Coord::new(1, 0)]);
        assert_eq!(turn.enemies()[0].head(), Coord::new(0, 2));
        assert_eq!(turn.enemies()[0].tail(), Coord::new(1, 3));
        assert_eq!(turn.you().head(), Coord::new(2, 1));
        assert_eq!(turn.you().tail(), Coord::new(2, 2));
    }

    // #[test]
    fn test_facing_wall() {
        let result = decide!("
        |  |  |  |
        |Y0|Y1|  |
        |  |  |  |
        ");
        assert_ne!(result, Some(Left)); //would hit wall
        assert_ne!(result, Some(Right)); //would hit self

        assert_eq!(Some(Down), decide!("
        |Y0|Y1|  |
        |  |  |  |
        |  |  |  |
        "));

    }

    // #[test]
    fn test_facing_self() {
        //could also go right to avoid tail, but would be trapped
        assert_eq!(Some(Left), decide!("
        |  |  |  |  |  |
        |Y8|Y7|Y6|Y5|  |
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        "));
    }

    // #[test]
    fn test_facing_other() {
        //should avoid being trapped between self and other snake
        assert_eq!(Some(Left), decide!("
        |A3|  |  |  |  |
        |A2|A1|A0|Y5|Y6|
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        "));

        //will be trapped, but there is nowhere else to go
        assert_eq!(Some(Right), decide!("
        |A2|  |Y6|  |
        |A1|A0|Y5|  |
        |Y0|  |Y4|  |
        |Y1|Y2|Y3|  |
        "));

        //the only winning move is not to play...
        assert_eq!(None, decide!("
        |  |  |A0|A1|
        |  |Y1|Y0|A2|
        |  |A5|A4|A3|
        |  |  |  |  |
        "));
    }

    // #[test]
    fn test_lookahead() {
        //looks like trapped, but actually next turn A's tail will move (assuming not stacked)
        assert_eq!(Some(Right), decide!("
        |  |A1|A2|A3|
        |  |A0|Y0|A4|
        |  |  |Y1|  |
        "));

        //going Up has more space now but is a dead end, while B's tail will move and open up space
        assert_eq!(Some(Right), decide!("
        |B0|  |  |  |  |  |  |
        |B1|B2|B3|B4|B5|  |  |
        |  |  |  |  |B6|B7|  |
        |A3|A2|A1|Y0|  |B8|  |
        |A4|A5|A0|Y1|C4|C3|  |
        |A7|A6|  |Y1|  |C2|  |
        |A8|A9|  |Y1|C0|C1|  |
        "));
    }
}
