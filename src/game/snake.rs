use crate::api::{ApiSnake, ApiDirection};
use super::{Offset, Path, Coord};

pub type Health = u8; //make this u16 if there will be health > 256
pub type ShortId = u8;

pub struct Snake {
    pub id: ShortId,
    pub health: Health,
    pub body: Path,
}

impl Snake {

    pub fn init(id: ShortId, api_snake: &ApiSnake) -> Option<Snake> {
        if let Some(body) = Path::init(&api_snake.body) {
            Option::from(Snake {
                id,
                health: api_snake.health as Health,
                body,
            })
        } else {
            Option::None
        }
    }

    //todo: why does this work without Coord implementing Copy/Clone?
    pub fn head(&self) -> Coord {
        self.body.start()
    }

    pub fn tail(&self) -> Coord {
        self.body.end()
    }

    pub fn starved(&self) -> bool {
        self.health == 0
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
        //todo: only allow legal moves
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
