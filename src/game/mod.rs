mod coord;
mod offset;
mod path;
mod snake;

//todo: is there some stuff we don't need to re-export?
use std::collections::HashMap;
use crate::api::{ApiSnakeId, ApiGameState, ApiDirection};
pub use coord::*;
pub use offset::*;
pub use path::*;
pub use snake::*;

const SNAKE_MAX_HEALTH: Health = 100;

//todo: can state be broken up in a way that allows memoization, avoiding cycles?
//todo: store probabilities and scores in the structure; update when invalidated?

pub struct NextTurn {
    you_move: ApiDirection,
    enemy_moves: Vec<ApiDirection>,
    result: Box<Turn>,
}

pub struct Turn {
    pub you: Snake,
    pub enemies: Vec<Snake>,
    pub food: Vec<Coord>,
    pub next: Option<Vec<NextTurn>>
}

impl Turn {
    fn init(game_state: &ApiGameState) -> Turn {
        Turn {
            you: Snake::init(&game_state.you),
            enemies: game_state.board.snakes.iter().map(|api_snake| Snake::init(api_snake)).collect(),
            food: game_state.board.food.iter().map(|&c| Coord::init(c)).collect(),
            next: None,
        }
    }

    fn find_food(&self, coord: Coord) -> Option<usize> {
        self.food.iter().position(|&food| food == coord)
    }

    //todo: these rules seem to conflict about order
    //https://docs.battlesnake.com/rules
    //https://github.com/BattlesnakeOfficial/rules/blob/master/standard.go
    //https://github.com/BattlesnakeOfficial/engine/blob/master/rules/tick.go
    pub fn advance(&mut self, you_move: ApiDirection, enemy_moves: &[ApiDirection], bound: Coord) {
        self.you.slither(you_move);
        if let Some(head) = self.you.head() {
            if let Some(food_index) = self.find_food(head) {
                self.food.remove(food_index);
                self.you.feed(SNAKE_MAX_HEALTH);
            }
        }

        for enemy_index in 0..self.enemies.len() {
            if let Some(&dir) = enemy_moves.get(enemy_index) {
                self.enemies[enemy_index].slither(dir);
                if let Some(head) = self.enemies[enemy_index].head() {
                    if let Some(food_index) = self.find_food(head) {
                        self.food.remove(food_index);
                        self.enemies[enemy_index].feed(SNAKE_MAX_HEALTH);
                    }
                }
            }
        }
    }
}

//persistent info about enemy snakes that doesn't vary turn-to-turn
struct EnemyData {
    pub name: String,
    //todo: try modeling enemy behaviour as a simple markov chain
}

pub struct Game {
    pub width: u32,
    pub height: u32,
    pub turn: Turn,
    enemy_data: HashMap<ApiSnakeId, EnemyData>,
}

impl Game {
    pub fn init(game_state: &ApiGameState) -> Game {
        Game {
            width: game_state.board.width,
            height: game_state.board.height,
            turn: Turn::init(game_state),
            enemy_data: HashMap::new(),
        }
    }

    pub fn update(&mut self, game_state: &ApiGameState) {
        //don't really expect this to change, but just in case!
        self.width = game_state.board.width;
        self.height = game_state.board.height;

        //todo: copy over `next` turn data if it was available and accurate
        self.turn = Turn::init(game_state);
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

        let turn = &game.turn;
        assert_eq!(turn.food, vec![Coord::new(1, 0)]);
        assert_eq!(turn.enemies[0].head().unwrap(), Coord::new(0, 2));
        assert_eq!(turn.enemies[0].tail().unwrap(), Coord::new(1, 3));
        assert_eq!(turn.you.head().unwrap(), Coord::new(2, 1));
        assert_eq!(turn.you.tail().unwrap(), Coord::new(2, 2));
    }
}
