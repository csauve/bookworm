mod coord;
mod offset;
mod path;
mod snake;

//todo: is there some stuff we don't need to re-export?
use crate::api::ApiGameState;
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

type NextTurns = Option<Vec<FutureTurn>>;

pub struct FutureTurn {
    pub turn: Turn,
    pub next: NextTurns,
}

pub struct Game {
    pub width: u32,
    pub height: u32,
    pub history: Vec<Option<Turn>>, //option in case we miss some calls
    pub future: NextTurns,
}

impl Turn {
    pub fn init(game_state: &ApiGameState) -> Turn {
        //todo
        Turn {
            index: game_state.turn as usize,
            you: Snake::init(you_id, &game_state.you),
            enemies: game_state.board.snakes.clone(),
            food: game_state.board.food.clone(),
        }
    }
}

impl Game {
    pub fn init(game_state: &ApiGameState) -> Game {
        let mut game = Game {
            width: game_state.board.width,
            height: game_state.board.height,
            history: Vec::new(),
            future: Option::None,
        };
        game.update(game_state);
        game
    }

    //todo
    pub fn update(&mut self, game_state: &ApiGameState) {
        let new_turn = Turn::init(game_state);
        if self.history.len() == new_turn.index {
            self.history.push(Option::from(new_turn));
        } else if self.history.len() < new_turn.index {
            //history buffer isn't long enough to hold this turn
            // self.history.extend_with(n: usize, mut value: E)
        } else {
            //the new turn occurs somewhere in history
            self.history[new_turn.index] = Option::from(new_turn);
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
