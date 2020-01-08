use crate::api::{ApiSnake, ApiDirection};
use super::{Offset, Path, Coord, ZERO};

pub type Health = u8; //make this u16 if there will be health > 256

#[derive(Clone)]
pub struct Snake {
    pub health: Health,
    body: Path,
}

impl Snake {

    pub fn from_api(api_snake: &ApiSnake) -> Snake {
        Snake {
            health: api_snake.health as Health,
            body: Path::from_api(&api_snake.body),
        }
    }

    pub fn head(&self) -> Option<Coord> {
        self.body.start()
    }

    pub fn tail(&self) -> Option<Coord> {
        self.body.end()
    }

    pub fn starved(&self) -> bool {
        self.health == 0
    }

    pub fn self_collided(&self) -> bool {
        self.body.start_self_intersects()
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
}

//todo: write tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake() {

    }
}
