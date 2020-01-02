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
    fn test_at_wall() {
        let result = decide!("
        |  |  |  |
        |Y0|Y1|  |
        |  |  |  |
        ");

        assert_ne!(result, ApiDirection::Left); //would hit wall
        assert_ne!(result, ApiDirection::Right); //would hit self
    }
}
