use std::iter::once;
use std::convert::TryFrom;
use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use rayon::prelude::*;
use crate::api::{ApiGameState, ApiDirection};
use super::util::cartesian_product;
use super::snake::{Snake, Health};
use super::coord::{Coord, Unit, UnitAbs};
use super::path::Path;

const SNAKE_MAX_HEALTH: Health = 100;
const ORIGIN: Coord = Coord {x: 0, y: 0};
const ALL_DIRS: [ApiDirection; 4] = [ApiDirection::Down, ApiDirection::Left, ApiDirection::Up, ApiDirection::Right];

#[derive(PartialEq, Debug)]
pub enum AdvanceResult {
    YouLive, YouDie
}

#[derive(Clone, PartialEq, Debug)]
pub struct Turn {
    //must contain at least 1 snake (the `you` snake, at index 0)
    pub snakes: Vec<Snake>,
    pub food: Vec<Coord>,
    bound: Coord,
}

pub struct Territory {
    pub area: UnitAbs,
    pub food: Vec<Path>,
}

impl Turn {
    pub fn init(game_state: &ApiGameState) -> Turn {
        Turn {
            snakes: once(&game_state.you)
                .chain(game_state.board.snakes.iter())
                .map(|s| Snake::from_api(s).unwrap())
                .collect(),
            food: game_state.board.food.iter().map(Coord::from).collect(),
            bound: Coord::new(
                game_state.board.width as Unit - 1,
                game_state.board.height as Unit - 1
            ),
        }
    }

    //gets the set of moves from this point which are not obstructed or out of bounds
    pub fn get_free_moves(&self, from: Coord, n_turns: usize) -> Vec<ApiDirection> {
        ALL_DIRS.iter().cloned().filter(|dir| {
            let new_coord = from + (*dir).into();
            new_coord.bounded_by(ORIGIN, self.bound) && self.snakes.iter().all(|snake| {
                if (new_coord - snake.head()).manhattan_dist() > snake.size() {
                    //can save a little time ruling out snakes which are too far away
                    true
                } else if let Some(i) = snake.find_first_node(new_coord) {
                    //finding the "first" node is key to avoiding moving into stacked tail coords
                    //its safe to move into another snake if that node will be gone in n_turns
                    i >= snake.size().saturating_sub(n_turns)
                } else {
                    true
                }
            })
        }).collect()
    }

    //find out where each snake can move to next
    pub fn get_free_snake_moves(&self) -> Vec<Vec<ApiDirection>> {
        self.snakes.iter().map(|snake| self.get_free_moves(snake.head(), 1)).collect()
    }

    //A* pathfinding, taking into account snake tail movements
    pub fn pathfind(&self, from: Coord, to: Coord) -> Option<Path> {
        let mut frontier: HashSet<Coord> = HashSet::new();
        let mut breadcrumbs: HashMap<Coord, Coord> = HashMap::new();
        let mut best_dists: HashMap<Coord, UnitAbs> = HashMap::new();
        frontier.insert(from);
        best_dists.insert(from, 0);

        //todo: we spend a lot of time in min_by_key and hashing Coords -- optimize?
        while !frontier.is_empty() {
            let leader = *frontier.iter().min_by_key(|&coord| {
                best_dists.get(coord).unwrap() + (to - *coord).manhattan_dist()
            }).unwrap();

            if leader == to {
                let mut nodes = vec![leader];
                while let Some(prev) = breadcrumbs.get(nodes.last().unwrap()) {
                    nodes.push(*prev);
                }
                return Some(Path::from_vec(nodes));
            }

            frontier.remove(&leader);
            let leader_dist = *best_dists.get(&leader).unwrap();
            let free_spaces = self.get_free_moves(leader, leader_dist).iter()
                .map(|dir| leader + (*dir).into())
                .collect::<Vec<_>>();

            for free_space in free_spaces {
                let dist = leader_dist + 1;
                let best_dist = best_dists.get(&free_space);
                if best_dist.is_none() || dist < *best_dist.unwrap() {
                    breadcrumbs.insert(free_space, leader);
                    best_dists.insert(free_space, dist);
                    frontier.insert(free_space);
                }
            }
        }

        None
    }

    //for each coord on the board, find out which snake is closest. in a tie, neither snake receives the coord
    //todo: add a resolution parameter to do every n coords?
    pub fn get_territories(&self) -> Vec<Territory> {
        let territories = Mutex::new(self.snakes.iter().map(|_| {
            Territory {
                area: 0,
                food: Vec::new(),
            }
        }).collect::<Vec<_>>());

        let coords = cartesian_product(&[(0..self.bound.x).collect(), (0..self.bound.y).collect()]);

        //approx 5x speedup on 8-core system doing in parallel
        coords.par_iter().for_each(|coord| {
            let coord = Coord::new(coord[0], coord[1]);
            let mut best_path: Option<Path> = None;
            let mut best_snake: Option<usize> = None;

            //sort snakes by their best case distances -- it's likely we can skip checking most of them
            let mut sorted_snakes = self.snakes.iter()
                .enumerate()
                .map(|(i, snake)| (i, snake)) //make sure to include original index before sorting
                .collect::<Vec<_>>();
            sorted_snakes.sort_unstable_by_key(|(_, snake)| (coord - snake.head()).manhattan_dist());

            for (i, snake) in sorted_snakes.iter() {
                let snake_head = snake.head();

                //because snakes are sorted by best case distance, can finish early if we can't do any better
                if let Some(best) = best_path.as_ref() {
                    if (coord - snake_head).manhattan_dist() > best.dist() {
                        break;
                    }
                }

                if let Some(path) = self.pathfind(snake_head, coord) {
                    let path_dist = path.dist();
                    if best_path.is_none() || path_dist < best_path.as_ref().unwrap().dist() {
                        best_path = Some(path);
                        best_snake = Some(*i);
                    } else if path_dist == best_path.as_ref().unwrap().dist() {
                        best_snake = None;
                    }
                }
            }

            if let Some(i) = best_snake {
                let mut territories = territories.lock().unwrap();
                let territory = territories.get_mut(i).unwrap();
                territory.area += 1;
                if self.find_food(coord).is_some() {
                    territory.food.push(best_path.unwrap());
                }
            }
        });

        territories.into_inner().unwrap()
    }

    //Applies game rules to the turn in order to predict the result. Note that we can't predict food spawns.
    pub fn advance(&mut self, snake_moves: &[ApiDirection]) -> AdvanceResult {
        let mut eaten_food_indices: HashSet<usize> = HashSet::new();

        //move snakes and find eaten food
        for snake_index in 0..self.snakes.len() {
            let dir = snake_moves.get(snake_index).cloned().unwrap_or_else(|| self.snakes[snake_index].get_default_move());
            self.snakes[snake_index].slither(dir);
            if let Some(food_index) = self.find_food(self.snakes[snake_index].head()) {
                //all snakes get a chance to eat fairly before food is removed
                self.snakes[snake_index].feed(SNAKE_MAX_HEALTH);
                eaten_food_indices.insert(food_index);
            }
        }

        let dead_snake_indices = self.snakes.iter().enumerate().filter_map(|(snake_index, snake)| {
            if snake.starved() {
                return Some(snake_index);
            }
            if !snake.head().bounded_by(ORIGIN, self.bound) {
                return Some(snake_index);
            }
            for (other_snake_index, other_snake) in self.snakes.iter().enumerate() {
                if let Some(i) = other_snake.find_first_node(snake.head()) {
                    if i > 0 || (other_snake_index != snake_index && snake.size() <= other_snake.size()) {
                        //TWO SNAKES ENTER, ONE SNAKE LEAVES (Ok, actually neither may leave)
                        return Some(snake_index);
                    }
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
                .filter_map(|(i, s)| if dead_snake_indices.contains(&i) {None} else {Some(s.clone())})
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

    pub fn width(&self) -> UnitAbs {
        (self.bound.x + 1) as UnitAbs
    }

    pub fn height(&self) -> UnitAbs {
        (self.bound.y + 1) as UnitAbs
    }

    pub fn area(&self) -> UnitAbs {
        self.width() * self.height()
    }

    fn find_food(&self, coord: Coord) -> Option<usize> {
        self.food.iter().position(|&food| food == coord)
    }

    pub fn infer_you_move(prev_turn: &Turn, next_turn: &Turn) -> Result<ApiDirection, &'static str> {
        ApiDirection::try_from(next_turn.you().head() - prev_turn.you().head())
    }
}

//todo: write more tests (head-to-head, head-to-body, pathfinding, territories, free moves, ...)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiDirection::*;
    use super::AdvanceResult::*;

    macro_rules! advance {
        ($moves:expr, $curr:expr) => (
            {
                let game_state = ApiGameState::parse_basic($curr);
                let prev = Turn::init(&game_state);
                let mut next = prev.clone();
                let result = next.advance($moves);
                (prev, next, result)
            }
        );
    }

    #[test]
    fn test_init() {
        let api_game = ApiGameState::parse_basic("
        |  |()|  |
        |  |  |Y0|
        |A0|A1|Y1|
        |  |A2|  |
        |  |  |  |
        ");

        let turn = Turn::init(&api_game);
        assert_eq!(turn.food, vec![Coord::new(1, 0)]);
        assert_eq!(turn.enemies()[0].head(), Coord::new(0, 2));
        assert_eq!(turn.enemies()[0].tail(), Coord::new(1, 3));
        assert_eq!(turn.you().head(), Coord::new(2, 1));
        assert_eq!(turn.you().tail(), Coord::new(2, 2));
    }

    #[test]
    fn test_advance() {
        let (prev, next, result) = advance!(&[Up, Left], "
        |  |()|  |
        |  |  |Y0|
        |A0|A1|Y1|
        |  |A2|  |
        |  |  |  |
        ");

        //the Y snake didn't hit any walls
        assert_eq!(result, YouLive);
        //no food was eaten
        assert_eq!(prev.food, next.food);
        assert_eq!(prev.food, next.food);
        //health of snakes goes down each turn
        assert_eq!(next.you().health, prev.you().health - 1);
        //snake A hit a wall
        assert!(next.enemies().is_empty());
        //moved Y snake according to intended direction
        assert_eq!(next.you().head(), Coord::new(2, 0));
    }

    #[test]
    fn test_you_die() {
        let (prev, next, result) = advance!(&[Up], "
        |  |  |  |  |  |
        |Y8|Y7|Y6|Y5|  |
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        ");
        assert_eq!(prev.you().size(), 9);
        assert_eq!(next.you().size(), 9);
        assert_eq!(result, YouDie);
    }
}
