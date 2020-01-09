mod coord;
mod offset;
mod path;
mod snake;

//todo: is there some stuff we don't need to re-export?
use std::collections::HashMap;
use std::iter::once;
use crate::api::{ApiSnakeId, ApiGameState, ApiDirection};
pub use coord::*;
pub use offset::*;
pub use path::*;
pub use snake::*;

const SNAKE_MAX_HEALTH: Health = 100;
const ORIGIN: Coord = Coord {x: 0, y: 0};

//todo: can state be broken up in a way that allows memoization, avoiding cycles?
//todo: store probabilities and scores in the structure; update when invalidated?

pub struct NextTurn {
    you_move: ApiDirection,
    enemy_moves: Vec<ApiDirection>,
    result: Box<Turn>,
}

//must contain at least 1 snake (the `you` snake, at index 0)
pub struct Turn {
    pub snakes: Vec<Snake>,
    pub food: Vec<Coord>,
    pub next: Option<Vec<NextTurn>>
}

impl Turn {
    fn init(game_state: &ApiGameState) -> Turn {
        Turn {
            //todo: isnt this a move out of borrow???
            snakes: once(game_state.you).chain(game_state.board.snakes).map(|s| Snake::from_api(&s)).collect(),
            food: game_state.board.food.iter().map(Coord::from).collect(),
            next: None,
        }
    }

    pub fn you(&self) -> &Snake {
        self.snakes.first().unwrap()
    }

    pub fn enemies(&self) -> &[Snake] {
        if self.snakes.len() > 1 {
            &self.snakes[1..]
        } else {
            &[]
        }
    }

    fn find_food(&self, coord: Coord) -> Option<usize> {
        self.food.iter().position(|&food| food == coord)
    }

    //https://docs.battlesnake.com/rules
    //https://github.com/BattlesnakeOfficial/rules/blob/master/standard.go
    //https://github.com/BattlesnakeOfficial/engine/blob/master/rules/tick.go
    pub fn tick(&mut self, snake_moves: &[ApiDirection], bound: Coord) -> Result<Ok, &'static str> {
        let mut eaten_food: Vec<Coord> = Vec::new();

        for snake_index in 0..self.snakes.len() {
            let dir = *snake_moves.get(snake_index).unwrap_or(&self.snakes[snake_index].get_default_move());
            self.snakes[snake_index].slither(dir);
            if let Some(head) = self.snakes[snake_index].head() {
                if let Some(food_index) = self.find_food(head) {
                    eaten_food.push(self.food[food_index]);
                    self.snakes[snake_index].feed(SNAKE_MAX_HEALTH);
                }
            }
        }

        //all snakes get a chance to eat fairly before food is removed
        if !eaten_food.is_empty() {
            self.food = self.food.iter()
                .filter_map(|f| if eaten_food.contains(f) {None} else {Some(*f)})
                .collect();
        }

        //https://github.com/BattlesnakeOfficial/engine/blob/8943c2c4e8777c39c5d20b82d097e229bc6c850a/rules/death.go#L13
        let dead_snakes = self.snakes.iter().enumerate().filter_map(|(snake_index, snake)| {
            if snake.starved() {
                return Some(snake_index);
            }
            if let Some(head) = snake.head() {
                if !head.bounded_by(ORIGIN, bound) {
                    return Some(snake_index);
                }
                for (enemy_index, enemy) in self.snakes.iter().enumerate() {
                    //todo: head-to-body collisions and self-collisions

                    //TWO SNAKE ENTER; ONE SNAKE LEAVE
                    if enemy_index != snake_index {
                        if let Some(enemy_head) = enemy.head() {
                            if head == enemy_head && snake.size() <= enemy.size() {
                                return Some(snake_index);
                            }
                        }
                    }
                }
                return None;
            }
            Some(snake_index) //headless
        }).collect::<Vec<_>>();

        //the loop which created dead_snakes would have produced an ordered list of indices, so
        //we can just use swap_remove in reverse to filter the snakes in-place since we don't
        //care about preserving order. This should just be O(dead_snakes)
        for dead_snake_index in dead_snakes.iter().rev() {
            if *dead_snake_index == 0 {
                //the `you` snake died...
            }
            self.snakes.swap_remove(*dead_snake_index);
        }
    }
}

//persistent info about enemy snakes that doesn't vary turn-to-turn
struct EnemyData {
    pub name: String,
    //todo: try modeling enemy behaviour as a simple markov chain
}

pub struct Game {
    bound: Coord,
    pub turn: Turn,
    enemy_data: HashMap<ApiSnakeId, EnemyData>,
}

impl Game {
    pub fn init(game_state: &ApiGameState) -> Game {
        Game {
            bound: Coord::new(
                game_state.board.width as Unit - 1,
                game_state.board.height as Unit - 1
            ),
            turn: Turn::init(game_state),
            enemy_data: HashMap::new(),
        }
    }

    pub fn update(&mut self, game_state: &ApiGameState) {
        //don't really expect this to change, but just in case!
        self.bound = Coord::new(
            game_state.board.width as Unit - 1,
            game_state.board.height as Unit - 1
        );

        let prev_turn = &self.turn;
        let new_turn = Turn::init(game_state);

        //todo: copy over `next` turn data if it was available and accurate

        self.turn = new_turn;
    }

    pub fn width(&self) -> Unit {
        self.bound.x + 1
    }

    pub fn height(&self) -> Unit {
        self.bound.y + 1
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

        assert_eq!(game.height(), 5);
        assert_eq!(game.width(), 3);

        let turn = &game.turn;
        assert_eq!(turn.food, vec![Coord::new(1, 0)]);
        assert_eq!(turn.enemies()[0].head().unwrap(), Coord::new(0, 2));
        assert_eq!(turn.enemies()[0].tail().unwrap(), Coord::new(1, 3));
        assert_eq!(turn.you().head().unwrap(), Coord::new(2, 1));
        assert_eq!(turn.you().tail().unwrap(), Coord::new(2, 2));
    }
}
