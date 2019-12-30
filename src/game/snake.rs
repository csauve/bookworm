use crate::api::ApiSnake;
use super::coord::Coord;
use super::path::Path;
use std::cmp::min;

//todo: using Path here means we don't know true length (e.g. stacked ApiCoords). Problem?
pub struct Snake {
    pub id: u8,
    health: u8,
    pub body: Path,
}

impl Snake {

    pub fn init(id: u8, api_snake: &ApiSnake) -> Snake {
        Snake {
            id,
            //cap at 256 instead of wrapping if health from API was higher
            health: min(api_snake.health, std::u8::MAX as u32) as u8,
            body: Path::init(&api_snake.body),
        }
    }

    //todo: why does this work without Coord implementing Copy/Clone?
    pub fn head(&self) -> Coord {
        self.body.end()
    }

    pub fn tail(&self) -> Coord {
        self.body.start()
    }

    pub fn starved(&self) -> bool {
        self.health == 0
    }

    //todo: mutation fns like move()
}

//todo: write tests
