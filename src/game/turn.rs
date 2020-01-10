use std::iter::once;
use std::convert::TryFrom;
use std::collections::HashSet;
use crate::api::{ApiGameState, ApiDirection};
use super::snake::{Snake, Health};
use super::coord::Coord;

const SNAKE_MAX_HEALTH: Health = 100;
const ORIGIN: Coord = Coord {x: 0, y: 0};

pub enum AdvanceResult {
    YouLive, YouDie
}

pub struct Turn {
    //must contain at least 1 snake (the `you` snake, at index 0)
    pub snakes: Vec<Snake>,
    pub food: Vec<Coord>,
    pub next: Option<Vec<Turn>>
}

impl Turn {
    fn init(game_state: &ApiGameState) -> Turn {
        Turn {
            //todo: isnt this a move out of borrow???
            snakes: once(game_state.you)
                .chain(game_state.board.snakes)
                .map(|s| Snake::from_api(&s).unwrap())
                .collect(),
            food: game_state.board.food.iter().map(Coord::from).collect(),
            next: None,
        }
    }

    fn update(&mut self, game_state: &ApiGameState) {
        self.snakes = once(game_state.you)
            .chain(game_state.board.snakes)
            .map(|s| Snake::from_api(&s).unwrap())
            .collect();
        self.food = game_state.board.food.iter().map(Coord::from).collect();
        self.next = self.next.map(|turns| {
            //todo: need to find matching turn in next turns, then use its `next`
            turns
        });
    }

    //Applies game rules to the turn in order to predict the result. Note that we can't predict food spawns.
    pub fn advance(&mut self, snake_moves: &[ApiDirection], bound: Coord) -> AdvanceResult {
        //all snakes get a chance to eat fairly before food is removed
        let eaten_food_indices = self.snakes.iter_mut().enumerate().filter_map(|(snake_index, snake)| {
            let dir = *snake_moves.get(snake_index).unwrap_or(&snake.get_default_move());
            snake.slither(dir);
            let found_food = self.find_food(self.snakes[snake_index].head());
            if found_food.is_some() {
                snake.feed(SNAKE_MAX_HEALTH);
            }
            found_food
        }).collect::<HashSet<usize>>();

        let dead_snake_indices = self.snakes.iter().enumerate().filter_map(|(snake_index, snake)| {
            if snake.starved() {
                return Some(snake_index);
            }
            if !snake.head().bounded_by(ORIGIN, bound) {
                return Some(snake_index);
            }
            for (other_snake_index, other_snake) in self.snakes.iter().enumerate() {
                //short circuit on body hits since probably more likely than head-to-head
                if snake.hit_body_of(other_snake) || other_snake_index != snake_index && snake.loses_head_to_head(other_snake) {
                    //TWO SNAKES ENTER, ONE SNAKE LEAVES
                    return Some(snake_index);
                }
            }
            None
        }).collect::<HashSet<usize>>();

        if dead_snake_indices.contains(&0) {
            return AdvanceResult::YouDie;
        }

        //clean up
        if !eaten_food_indices.is_empty() {
            self.food = self.food.iter().enumerate()
                .filter_map(|(i, f)| if eaten_food_indices.contains(&i) {None} else {Some(*f)})
                .collect();
        }
        if !dead_snake_indices.is_empty() {
            self.snakes = self.snakes.iter().enumerate()
                .filter_map(|(i, s)| if dead_snake_indices.contains(&i) {None} else {Some(*s)})
                .collect();
        }
        AdvanceResult::YouLive
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

    pub fn infer_you_move(prev_turn: &Turn, next_turn: &Turn) -> Result<ApiDirection, &'static str> {
        ApiDirection::try_from(next_turn.you().head() - prev_turn.you().head())
    }
}
