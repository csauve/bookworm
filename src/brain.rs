use crate::game::Game;
use crate::api::ApiDirection;

pub fn get_decision(_game: &Game) -> ApiDirection {
    //todo
    ApiDirection::Up
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
        assert_ne!(result, ApiDirection::Left); //would hit wall
        assert_ne!(result, ApiDirection::Right); //would hit self

        assert_eq!(ApiDirection::Down, decide!("
        |Y0|Y1|  |
        |  |  |  |
        |  |  |  |
        "));

    }

    #[test]
    fn test_facing_self() {
        //could also go right to avoid tail, but would be trapped
        assert_eq!(ApiDirection::Left, decide!("
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
        assert_eq!(ApiDirection::Left, decide!("
        |A3|  |  |  |  |
        |A2|A1|A0|Y5|Y6|
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        "));

        //will be trapped, but there is nowhere else to go
        assert_eq!(ApiDirection::Right, decide!("
        |A2|  |Y6|  |
        |A1|A0|Y5|  |
        |Y0|  |Y4|  |
        |Y1|Y2|Y3|  |
        "));

        //we are already dead... charge!!!
        assert_eq!(ApiDirection::Right, decide!("
        |  |  |A0|A1|
        |  |Y1|Y0|A2|
        |  |A5|A4|A3|
        |  |  |  |  |
        "));
    }
}
