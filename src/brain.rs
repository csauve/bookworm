use crate::game::Game;
use crate::api::ApiDirection;

pub fn get_decision(game: &Game) -> ApiDirection {
    //todo
    ApiDirection::Up
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ApiGameState};

    #[test]
    fn test_at_wall() {
        let result = get_decision(&Game::init(&ApiGameState::parse_basic("
        |  |  |  |\
        |Y0|Y1|  |\
        |  |  |  |\
        ")));

        assert_ne!(result, ApiDirection::Left); //would hit wall
        assert_ne!(result, ApiDirection::Right); //would hit self
    }
}
