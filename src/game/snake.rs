use crate::api::{ApiSnake, ApiDirection};
use super::{Offset, Path, Coord, ZERO};

pub type Health = u8; //make this u16 if there will be health > 256
pub type ShortId = u8;

pub struct Snake {
    pub id: ShortId,
    pub health: Health,
    body: Path,
}

impl Snake {

    pub fn init(id: ShortId, api_snake: &ApiSnake) -> Snake {
        Snake {
            id,
            health: api_snake.health as Health,
            body: Path::init(&api_snake.body),
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
        let offset = match dir {
            ApiDirection::Left => Offset::new(-1, 0),
            ApiDirection::Right => Offset::new(1, 0),
            ApiDirection::Up => Offset::new(0, -1),
            ApiDirection::Down => Offset::new(0, 1),
        };
        self.body.slide_start(offset);
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
