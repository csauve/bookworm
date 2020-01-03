use crate::game::Game;
use crate::api::ApiDirection;
use crate::api::ApiDirection::*;

pub fn get_decision(_game: &Game) -> Option<ApiDirection> {
    //todo
    Some(Up)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ApiGameState};

    macro_rules! decide {
        ($s:expr) => (get_decision(&Game::init(&ApiGameState::parse_basic($s))));
    }

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
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
