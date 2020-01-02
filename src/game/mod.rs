mod coord;
mod offset;
mod path;
mod snake;

//todo: is there some stuff we don't need to re-export?
use std::collections::HashMap;
use std::cmp::Ordering::*;
use crate::api::{ApiSnakeId, ApiGameState};
pub use coord::*;
pub use offset::*;
pub use path::*;
pub use snake::*;

//todo: can state be broken up in a way that allows memoization, avoiding cycles?
//todo: store probabilities and scores in the structure; update when invalidated?

pub struct Turn {
    pub you: Snake,
    pub enemies: Vec<Snake>,
    pub food: Vec<Coord>,
}

impl Turn {
    fn init(snake_indexes: &HashMap<ApiSnakeId, u8>, game_state: &ApiGameState) -> Turn {
        Turn {
            you: Snake::init(snake_indexes[&game_state.you.id], &game_state.you)
                .expect("API game state contained invalid `you` snake. Wat do!?"),
            enemies: game_state.board.snakes.iter().filter_map(|api_snake| {
                Snake::init(snake_indexes[&api_snake.id], api_snake)
            }).collect(),
            food: game_state.board.food.iter().map(|c| Coord::init(*c)).collect(),
        }
    }
}

//persistent info about snakes that doesn't vary turn-to-turn
struct SnakeData {
    pub name: String,
    pub api_id: ApiSnakeId,
}

pub struct Game {
    pub width: u32,
    pub height: u32,
    pub history: Vec<Option<Turn>>, //option in case we miss some calls
    snake_data: Vec<SnakeData>,
}

impl Game {
    pub fn init(game_state: &ApiGameState) -> Game {
        let mut game = Game {
            width: game_state.board.width,
            height: game_state.board.height,
            history: Vec::new(),
            snake_data: Vec::new(),
        };
        game.update(game_state);
        game
    }

    pub fn update(&mut self, game_state: &ApiGameState) {

        //todo: include the `you` snake and self-initialize the `snake_data`
        let snake_indexes = game_state.board.snakes.iter().map(|api_snake| {
            let index = self.snake_data.iter().position(|snake_data| {
                snake_data.api_id == api_snake.id
            }).expect("Unexpected snake ID");
            (api_snake.id.clone(), index as u8)
        }).collect::<HashMap<_, _>>();

        let new_turn = Turn::init(&snake_indexes, game_state);
        let new_turn_index = game_state.turn as usize;

        match self.history.len().cmp(&new_turn_index) {
            Equal => {
                self.history.push(Option::from(new_turn));
            },
            Less => {
                //history buffer isn't long enough to hold this turn
                self.history.resize_with(new_turn_index + 1, || Option::None);
                self.history[new_turn_index] = Option::from(new_turn);
            },
            Greater => {
                //the new turn occurs somewhere in history
                self.history[new_turn_index] = Option::from(new_turn);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::*;

    #[test]
    fn test_init() {
        let api_game = ApiGameState::parse_basic("\
        |  |()|  |
        |  |  |Y0|
        |A0|A1|Y1|
        |  |A2|  |
        |  |  |  |
        ");

        let game = Game::init(&api_game);

        assert_eq!(game.height, 5);
        assert_eq!(game.width, 3);
        assert_eq!(game.history.len(), 3);
        assert!(game.history[0].is_none());
        assert!(game.history[1].is_none());

        let turn: &Turn = game.history[2].as_ref().unwrap();
        assert_eq!(turn.food, vec![Coord::new(1, 0)]);
        assert_eq!(turn.enemies[0].head(), Coord::new(0, 2));
        assert_eq!(turn.enemies[0].tail(), Coord::new(1, 3));
        assert_eq!(turn.you.head(), Coord::new(2, 1));
        assert_eq!(turn.you.tail(), Coord::new(2, 2));
    }
}
