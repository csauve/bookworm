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
    pub index: usize,
    pub you: Snake,
    pub enemies: Vec<Snake>,
    pub food: Vec<Coord>,
}

impl Turn {
    pub fn init(snake_indexes: &HashMap<ApiSnakeId, u8>, game_state: &ApiGameState) -> Turn {
        Turn {
            index: game_state.turn as usize,
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
            snake_data: game_state.board.snakes.iter().map(|api_snake| {
                SnakeData {
                    name: api_snake.name.clone(),
                    api_id: api_snake.id.clone(),
                }
            }).collect(),
        };
        game.update(game_state);
        game
    }

    pub fn update(&mut self, game_state: &ApiGameState) {
        let snake_indexes = game_state.board.snakes.iter().map(|api_snake| {
            let index = self.snake_data.iter().position(|snake_data| {
                snake_data.api_id == api_snake.id
            }).expect("Unexpected snake ID");
            (api_snake.id.clone(), index as u8)
        }).collect::<HashMap<_, _>>();
        let new_turn = Turn::init(&snake_indexes, game_state);

        match self.history.len().cmp(&new_turn.index) {
            Equal => {
                self.history.push(Option::from(new_turn));
            },
            Less => {
                //history buffer isn't long enough to hold this turn
                let turn_index = new_turn.index;
                let new_len = turn_index + 1;
                self.history.resize_with(new_len, || Option::None);
                self.history[turn_index] = Option::from(new_turn);
            },
            Greater => {
                //the new turn occurs somewhere in history
                let turn_index = new_turn.index;
                self.history[turn_index] = Option::from(new_turn);
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
        let game_state = ApiGameState {
            game: ApiGame {
                id: ApiGameId::from("123"),
            },
            turn: 2,
            board: ApiBoard {
                height: 10,
                width: 10,
                food: vec![ApiCoords {x: 0, y: 0}],
                snakes: vec![
                    //todo: write some helper fns to spawn snakes
                ],
            },
            you: ApiSnake {
                id: ApiSnakeId::from("456"),
                name: String::from("snek"),
                health: 100,
                body: vec![ApiCoords {x: 1, y: 1}]
            }
        };
    }
}
