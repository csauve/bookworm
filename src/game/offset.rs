use std::ops::{Add, Sub};
use super::Coord;

pub const ZERO: Offset = Offset {
    dx: 0,
    dy: 0,
};

#[derive(Copy, Clone, PartialEq)]
pub struct Offset {
    pub dx: i16,
    pub dy: i16,
}

impl Offset {

    pub fn between(a: Coord, b: Coord) -> Offset {
        Offset {
            dx: b.x as i16 - a.x as i16,
            dy: b.y as i16 - a.y as i16,
        }
    }

    pub fn linear(&self) -> bool {
        self.dx == 0 || self.dy == 0
    }

    pub fn manhattan_dist(&self) -> u16 {
        self.dx.abs() as u16 + self.dy.abs() as u16
    }
}

impl Add for Offset {
    type Output = Offset;

    //todo: why is this `self` and not `&self`?
    fn add(self, rhs: Offset) -> Offset {
        Offset {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}

impl Sub for Offset {
    type Output = Offset;

    fn sub(self, rhs: Offset) -> Offset {
        Offset {
            dx: self.dx - rhs.dx,
            dy: self.dy - rhs.dy,
        }
    }
}

//todo: write tests
