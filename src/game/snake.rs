use std::convert::TryFrom;
use crate::api::{ApiSnake, ApiDirection};
use super::{Path, Coord, ZERO};

pub type Health = u8; //make this u16 if there will be health > 256

#[derive(Clone)]
pub struct Snake {
    pub health: Health,
    body: Path,
}

impl Snake {

    pub fn from_api(api_snake: &ApiSnake) -> Result<Snake, &'static str> {
        if api_snake.body.is_empty() {
            return Err("Malformed snake: body is empty")
        }
        Ok(Snake {
            health: api_snake.health as Health,
            body: Path::from_api(&api_snake.body),
        })
    }

    pub fn get_default_move(&self) -> ApiDirection {
        if let Some(neck) = self.neck() {
            if let Ok(dir) = ApiDirection::try_from(self.head() - neck) {
                return dir;
            }
        }
        ApiDirection::Up
    }

    pub fn head(&self) -> Coord {
        self.body.start().unwrap()
    }

    pub fn neck(&self) -> Option<Coord> {
        self.body.get_node(1)
    }

    pub fn tail(&self) -> Coord {
        self.body.end().unwrap()
    }

    pub fn starved(&self) -> bool {
        self.health == 0
    }

    pub fn hit_body_of(&self, other: &Snake) -> bool {
        other.body.contains_node(self.head()) && other.head() != self.head()
    }

    pub fn loses_head_to_head(&self, other: &Snake) -> bool {
        self.head() == other.head() && self.size() <= other.size()
    }

    pub fn feed(&mut self, new_health: Health) {
        self.health = new_health;
        self.body.extend_end(ZERO);
    }

    pub fn slither(&mut self, dir: ApiDirection) {
        if !self.starved() {
            self.health -= 1;
        }
        self.body.slide_start(dir.into());
    }

    pub fn size(&self) -> usize {
        self.body.num_nodes()
    }
}

//todo: write tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake() {

    }
}
