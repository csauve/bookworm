mod coord;
mod offset;
mod path;
mod snake;

//todo: is there some stuff we don't need to re-export?
use std::collections::HashMap;
use std::cmp::Ordering::*;
use crate::api::{ApiSnakeId, ApiGameState, ApiDirection};
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
    fn init(snake_data: &HashMap<ApiSnakeId, SnakeData>, game_state: &ApiGameState) -> Turn {
        Turn {
            you: Snake::init(0, &game_state.you)
                .expect("API game state contained invalid `you` snake. Wat do!?"),
            enemies: game_state.board.snakes.iter().filter_map(|api_snake| {
                Snake::init(snake_data[&api_snake.id].short_id, api_snake)
            }).collect(),
            food: game_state.board.food.iter().map(|c| Coord::init(*c)).collect(),
        }
    }

    //https://docs.battlesnake.com/rules
    pub fn advance(&mut self, moves: HashMap<u8, ApiDirection>, bound: Coord) {
        //todo
    }
}

//persistent info about snakes that doesn't vary turn-to-turn
struct SnakeData {
    pub name: String,
    pub short_id: u8,
}

pub struct Game {
    pub width: u32,
    pub height: u32,
    pub turns: Vec<Option<Turn>>, //option in case we miss some calls
    snake_data: HashMap<ApiSnakeId, SnakeData>,
}

impl Game {
    pub fn init(game_state: &ApiGameState) -> Game {
        let mut snake_data = HashMap::new();
        snake_data.insert(game_state.you.id.clone(), SnakeData {
            name: game_state.you.name.clone(),
            short_id: 0,
        });

        let mut game = Game {
            width: game_state.board.width,
            height: game_state.board.height,
            turns: Vec::new(),
            snake_data,
        };
        //populate the remaining snake data and the first turn
        game.update(game_state);
        game
    }

    pub fn update(&mut self, game_state: &ApiGameState) {
        //assign short IDs to snakes if they don't have one already
        for api_snake in game_state.board.snakes.iter() {
            if !self.snake_data.contains_key(&api_snake.id) {
                self.snake_data.insert(api_snake.id.clone(), SnakeData {
                    name: api_snake.name.clone(),
                    short_id: self.snake_data.values().map(|d| d.short_id).max().unwrap() + 1
                });
            }
        }

        let new_turn = Turn::init(&self.snake_data, game_state);
        let new_turn_index = game_state.turn as usize;

        match self.turns.len().cmp(&new_turn_index) {
            Equal => {
                self.turns.push(Option::from(new_turn));
            },
            Less => {
                //turns buffer isn't long enough to hold this turn
                self.turns.resize_with(new_turn_index + 1, || Option::None);
                self.turns[new_turn_index] = Option::from(new_turn);
            },
            Greater => {
                //the new turn occurs somewhere in turns
                self.turns[new_turn_index] = Option::from(new_turn);
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
        let api_game = ApiGameState::parse_basic("
        |  |()|  |
        |  |  |Y0|
        |A0|A1|Y1|
        |  |A2|  |
        |  |  |  |
        ");

        let game = Game::init(&api_game);

        assert_eq!(game.height, 5);
        assert_eq!(game.width, 3);
        assert_eq!(game.turns.len(), 1);
        assert!(game.turns[0].is_some());

        let turn: &Turn = game.turns[0].as_ref().unwrap();
        assert_eq!(turn.food, vec![Coord::new(1, 0)]);
        assert_eq!(turn.enemies[0].head(), Coord::new(0, 2));
        assert_eq!(turn.enemies[0].tail(), Coord::new(1, 3));
        assert_eq!(turn.you.head(), Coord::new(2, 1));
        assert_eq!(turn.you.tail(), Coord::new(2, 2));
    }
}
