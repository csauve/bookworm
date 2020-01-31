use std::iter;
use std::cmp::{Ord, Ordering, Eq, PartialEq, PartialOrd};
use std::collections::{HashSet, HashMap, BinaryHeap};
use std::sync::Mutex;
use std::fmt;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::iter::FromIterator;
use rayon::prelude::*;
use crate::api::{ApiGameState, ApiDirection};
use crate::util::cartesian_product;
use super::snake::{Snake, Health};
use super::coord::{Coord, Unit, UnitAbs};
use super::path::Path;
use log::*;

pub const SNAKE_MAX_HEALTH: Health = 100;
pub const SNAKE_START_SIZE: UnitAbs = 3;
const FOOD_SPAWN_CHANCE: u32 = 15; //of 100
const ORIGIN: Coord = Coord {x: 0, y: 0};
const ALL_DIRS: [ApiDirection; 4] = [ApiDirection::Down, ApiDirection::Left, ApiDirection::Up, ApiDirection::Right];
const PATHFINDING_HEURISTIC_WEIGHT: UnitAbs = 3;

#[derive(Clone, PartialEq, Debug)]
pub struct Board {
    //must contain at least 1 snake (the `you` snake, at index 0)
    pub snakes: Vec<Snake>,
    pub food: Vec<Coord>,
    bound: Coord,
}

pub struct Territory {
    pub area: UnitAbs,
    pub food: Vec<Path>,
}

#[derive(Eq)]
struct FrontierCoord(Coord, UnitAbs); //coord with f_score

impl Ord for FrontierCoord {
    //note: g_score tiebreaking did not have a benefit: https://movingai.com/astar.html
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1).reverse() //we want a min heap
    }
}

impl PartialOrd for FrontierCoord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FrontierCoord {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Board {
    pub fn init(width: UnitAbs, height: UnitAbs, num_snakes: usize) -> Result<Board, &'static str> {
        let mut rng = rand::thread_rng();
        let mut free_spaces: Vec<Coord> = Vec::from_iter(
            cartesian_product(&[
                (0..width as Unit).collect(),
                (0..height as Unit).collect()
            ]).iter().map(|v| Coord::new(v[0], v[1]))
        );

        let snakes = match (width, height) {
            (7, 7) | (11, 11) | (19, 19) if num_snakes <= 8 => {
                //the rules define 3 fixed board sizes with 8 fixed starting positions
                let mn = 1 as Unit;
                let md = ((width - 1) / 2) as Unit;
                let mx = (width - 2) as Unit;
                let mut fixed_starts = [
                    Coord::new(mn, mn),
                    Coord::new(mn, md),
                    Coord::new(mn, mx),
                    Coord::new(md, mn),
                    Coord::new(md, mx),
                    Coord::new(mx, mn),
                    Coord::new(mx, md),
                    Coord::new(mx, mx),
                ];
                fixed_starts.shuffle(&mut rng);
                fixed_starts.iter().take(num_snakes).map(|start| {
                    let i = free_spaces.iter().position(|coord| coord == start).unwrap();
                    free_spaces.swap_remove(i);
                    Snake::init(SNAKE_MAX_HEALTH, *start, SNAKE_START_SIZE)
                }).collect()
            },
            _ => {
                //otherwise, all snakes spawn in random positions if there's space
                if free_spaces.len() < num_snakes {
                    return Err("The board is not big enough to contain all requested snakes");
                }
                iter::repeat_with(|| {
                    let start = free_spaces.swap_remove(rng.gen_range(0, free_spaces.len()));
                    Snake::init(SNAKE_MAX_HEALTH, start, SNAKE_START_SIZE)
                }).take(num_snakes).collect()
            },
        };

        let food_spawner = iter::repeat_with(|| {
            if free_spaces.is_empty() {
                warn!("Ran out of free space to spawn food");
                None
            } else {
                Some(free_spaces.swap_remove(rng.gen_range(0, free_spaces.len())))
            }
        });

        Ok(Board {
            snakes,
            food: food_spawner
                .take_while(Option::is_some)
                .take(num_snakes)
                .map(Option::unwrap)
                .collect(),
            bound: Coord::new(
                width as Unit - 1,
                height as Unit - 1
            )
        })
    }

    pub fn from_api(game_state: &ApiGameState) -> Board {
        Board {
            snakes: iter::once(&game_state.you)
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

    //note -- hash key elements must are ordered such that key.0 < key.1
    //todo: this actually slowed down evaluate_board when used for pruning... can it be useful elsewhere?
    pub fn get_snake_dist_matrix(&self) -> HashMap<(usize, usize), UnitAbs> {
        let mut results = HashMap::new();
        for (a, snake_a) in self.snakes.iter().enumerate() {
            for (b, snake_b) in self.snakes.iter().enumerate().skip(a + 1) {
                let dist = (snake_a.head() - snake_b.head()).manhattan_dist();
                results.insert((a, b), dist);
            }
        }
        results
    }

    //gets the set of moves from this point which are not obstructed or out of bounds
    pub fn get_free_moves(&self, from: Coord, n_turns: usize) -> Vec<ApiDirection> {
        ALL_DIRS.iter().cloned().filter(|dir| {
            let new_coord = from + (*dir).into();
            new_coord.bounded_by(ORIGIN, self.bound) && self.snakes.iter().all(|snake| {
                if (new_coord - snake.head()).manhattan_dist() > snake.size() {
                    //can save a little time ruling out snakes which are too far away
                    true
                } else if let Some(i) = snake.find_first_node(new_coord, 0) {
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

    //todo: try returning distance only
    //A* pathfinding
    pub fn pathfind(&self, from: Coord, to: Coord) -> Option<Path> {
        //heap keeps open set sorted by best f_score
        let mut frontier: BinaryHeap<FrontierCoord> = BinaryHeap::new();
        //keeping known dists and breadcrumbs together in one tuple reduces hash operations
        let mut history: HashMap<Coord, (UnitAbs, Option<Coord>)> = HashMap::new();
        //static weighting: https://en.wikipedia.org/wiki/A*_search_algorithm#Bounded_relaxation
        frontier.push(FrontierCoord(from, (to - from).manhattan_dist() * PATHFINDING_HEURISTIC_WEIGHT));
        history.insert(from, (0, None));

        while let Some(FrontierCoord(leader, _leader_f_score)) = frontier.pop() {
            if leader == to {
                let mut nodes = vec![leader];
                while let Some((_, Some(prev))) = history.get(nodes.last().unwrap()) {
                    nodes.push(*prev);
                }
                return Some(Path::from_vec(nodes));
            }

            let leader_g_score = history.get(&leader).map(|hist| hist.0).unwrap_or(0);

            //use g_score as number of turns in the future so we can shorten snake tails
            let free_spaces = self.get_free_moves(leader, leader_g_score).iter()
                .map(|dir| leader + (*dir).into())
                .collect::<Vec<_>>();

            //todo: try JPS https://zerowidth.com/2013/a-visual-explanation-of-jump-point-search.html
            for free_space in free_spaces {
                let new_g_score = leader_g_score + 1;
                let old_g_score = history.get(&free_space);
                if old_g_score.is_none() || new_g_score < (*old_g_score.unwrap()).0 {
                    //todo: https://github.com/riscy/a_star_on_grids#avoid-recomputing-heuristics
                    history.insert(free_space, (new_g_score, Some(leader)));
                    let new_f_score = new_g_score + (to - free_space).manhattan_dist() * PATHFINDING_HEURISTIC_WEIGHT;
                    frontier.push(FrontierCoord(free_space, new_f_score));
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

        //large number of tasks with no need to synchronize; good spot to parallelize
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

    //Applies known game rules to the board, returning indices of snakes that died
    pub fn advance(&mut self, spawn_food: bool, snake_moves: &[ApiDirection]) -> HashSet<(usize, &'static str)> {
        let mut eaten_food: HashSet<usize> = HashSet::new();

        //move snakes and find eaten food
        for snake_index in 0..self.snakes.len() {
            let dir = snake_moves.get(snake_index).cloned().unwrap_or_else(|| self.snakes[snake_index].get_default_move());
            self.snakes[snake_index].slither(dir);
            if let Some(food_index) = self.find_food(self.snakes[snake_index].head()) {
                //all snakes get a chance to eat fairly before food is removed
                self.snakes[snake_index].feed(SNAKE_MAX_HEALTH);
                eaten_food.insert(food_index);
            }
        }

        let dead_snakes = self.snakes.iter().enumerate().filter_map(|(snake_index, snake)| {
            if snake.starved() {
                return Some((snake_index, "starved"));
            }
            if !snake.head().bounded_by(ORIGIN, self.bound) {
                return Some((snake_index, "out-of-bounds"));
            }
            for (other_snake_index, other_snake) in self.snakes.iter().enumerate() {
                if other_snake_index != snake_index {
                    if let Some(i) = other_snake.find_first_node(snake.head(), 0) {
                        if i > 0 {
                            return Some((snake_index, "other-collision"));
                        } else if snake.size() <= other_snake.size() {
                            //TWO SNAKES ENTER, ONE SNAKE LEAVES (Ok, actually neither may leave)
                            return Some((snake_index, "head-to-head"));
                        }
                    }
                } else if other_snake.find_first_node(snake.head(), 1).is_some() {
                    return Some((snake_index, "self-collision"));
                }
            }
            None
        }).collect::<HashSet<(usize, &'static str)>>();

        //clean up
        if !eaten_food.is_empty() {
            self.food = self.food.iter().enumerate()
                .filter_map(|(i, f)| if eaten_food.contains(&i) {None} else {Some(*f)})
                .collect();
        }
        if !dead_snakes.is_empty() {
            self.snakes = self.snakes.iter().enumerate()
                .filter_map(|(i, s)| {
                    if dead_snakes.iter().any(|(d, _)| *d == i) {
                        None
                    } else {
                        Some(s.clone())
                    }
                })
                .collect();
        }

        if spawn_food {
            let mut rng = rand::thread_rng();
            if rng.gen_range(0, 100) <= FOOD_SPAWN_CHANCE {
                let free_spaces: Vec<Coord> = Vec::from_iter(
                    cartesian_product(&[
                        (0..self.width() as Unit).collect(),
                        (0..self.height() as Unit).collect()
                    ]).iter().filter_map(|v| {
                        let coord = Coord::new(v[0], v[1]);
                        for snake in self.snakes.iter() {
                            if snake.find_first_node(coord, 0).is_some() {
                                return None;
                            }
                        }
                        Some(coord)
                    })
                );
                self.food.push(*free_spaces.get(rng.gen_range(0, free_spaces.len())).unwrap());
            }
        }

        dead_snakes
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

    //todo: Unit vs UnitAbs
    pub fn height(&self) -> UnitAbs {
        (self.bound.y + 1) as UnitAbs
    }

    pub fn area(&self) -> UnitAbs {
        self.width() * self.height()
    }

    fn find_food(&self, coord: Coord) -> Option<usize> {
        self.food.iter().position(|&food| food == coord)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "()")
    }
}

//todo: write more tests (head-to-head, head-to-body, pathfinding, territories, free moves, ...)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiDirection::*;

    macro_rules! advance {
        ($moves:expr, $curr:expr) => (
            {
                let game_state = ApiGameState::parse_basic($curr);
                let prev = Board::from_api(&game_state);
                let mut next = prev.clone();
                let result = next.advance(false, $moves);
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

        let board = Board::from_api(&api_game);
        assert_eq!(board.food, vec![Coord::new(1, 0)]);
        assert_eq!(board.enemies()[0].head(), Coord::new(0, 2));
        assert_eq!(board.enemies()[0].tail(), Coord::new(1, 3));
        assert_eq!(board.you().head(), Coord::new(2, 1));
        assert_eq!(board.you().tail(), Coord::new(2, 2));
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
        assert_eq!(result.len(), 1);
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
        assert_eq!(next.snakes.len(), 0);
        assert!(!result.is_empty());
    }
}
