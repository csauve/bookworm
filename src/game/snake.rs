use std::convert::TryFrom;
use std::iter;
use crate::api::{ApiSnake, ApiDirection};
use super::path::Path;
use super::coord::Coord;
use super::offset::ZERO as ZERO_OFFSET;

pub type Health = u8; //make this u16 if there will be health > 256

#[derive(Clone, PartialEq, Debug)]
pub struct Snake {
    pub health: Health,
    pub body: Path,
}

impl Snake {

    pub fn init(health: Health, start: Coord, size: usize) -> Snake {
        Snake {
            health,
            body: Path::from_vec(iter::repeat(start).take(size).collect())
        }
    }

    pub fn from_api(api_snake: &ApiSnake) -> Result<Snake, &'static str> {
        if api_snake.body.is_empty() {
            return Err("Malformed snake: body is empty")
        }
        Ok(Snake {
            health: api_snake.health as Health,
            body: Path::from_api(&api_snake.body),
        })
    }

    //default move according to battlesnake rules, used when a snake doesn't reply in time
    pub fn get_default_move(&self) -> ApiDirection {
        if let Some(neck) = self.neck() {
            if let Ok(dir) = ApiDirection::try_from(self.head() - neck) {
                return dir;
            }
        }
        ApiDirection::Up
    }

    //returns location of the snake's head, where movements are made from
    #[inline]
    pub fn head(&self) -> Coord {
        self.body.start().unwrap()
    }

    //returns the next distinct node after the head, if there is one
    pub fn neck(&self) -> Option<Coord> {
        self.body.get_node(1)
    }

    //returns the final node of the snake's body. may be its head if len 1
    pub fn tail(&self) -> Coord {
        self.body.end().unwrap()
    }

    #[inline]
    pub fn starved(&self) -> bool {
        self.health == 0
    }

    pub fn find_first_node(&self, loc: Coord) -> Option<usize> {
        self.body.nodes.iter().position(|node| loc == *node)
    }

    pub fn feed(&mut self, new_health: Health) {
        self.health = new_health;
        self.body.extend_end(ZERO_OFFSET);
    }

    pub fn slither(&mut self, dir: ApiDirection) {
        if !self.starved() {
            self.health -= 1;
        }
        self.body.slide_start(dir.into());
    }

    #[inline] //brings down pathfinding time a little bit
    pub fn size(&self) -> usize {
        self.body.num_nodes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiCoords;

    #[test]
    fn test_from_api_ok() {
        let snake = Snake::from_api(&ApiSnake {
            id: String::from("123abc"),
            name: String::from("waylon slithers"),
            health: 80,
            body: vec![
                ApiCoords {x: 1, y: 0},
                ApiCoords {x: 2, y: 0},
            ],
        });
        assert!(snake.is_ok());

        let snake = Snake::from_api(&ApiSnake {
            id: String::from("123abc"),
            name: String::from("waylon slithers"),
            health: 80,
            body: vec![],
        });
        assert!(snake.is_err());
    }

    #[test]
    fn test_snake_body() {
        let snake = Snake::from_api(&ApiSnake {
            id: String::from("123abc"),
            name: String::from("waylon slithers"),
            health: 80,
            body: vec![
                ApiCoords {x: 1, y: 0},
                ApiCoords {x: 2, y: 0},
                ApiCoords {x: 2, y: 1},
            ],
        }).unwrap();

        assert_eq!(snake.head(), Coord::new(1, 0));
        assert_eq!(snake.neck().unwrap(), Coord::new(2, 0));
        assert_eq!(snake.tail(), Coord::new(2, 1));
        assert_eq!(snake.get_default_move(), ApiDirection::Left);

        //unusual case, but should still work...
        let snake = Snake::from_api(&ApiSnake {
            id: String::from("123abc"),
            name: String::from("waylon slithers"),
            health: 80,
            body: vec![
                ApiCoords {x: 1, y: 0},
            ],
        }).unwrap();

        assert_eq!(snake.head(), Coord::new(1, 0));
        assert!(snake.neck().is_none());
        assert_eq!(snake.head(), Coord::new(1, 0));
        assert_eq!(snake.get_default_move(), ApiDirection::Up);
    }
}
