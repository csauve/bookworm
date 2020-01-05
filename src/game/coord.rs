use crate::api::ApiCoords;
use super::Offset;
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::cmp::{max, min};

//should change these if board will be bigger
pub type Unit = i8;
pub type UnitAbs = u8;

//todo: benchmark u8 vs i16
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coord {
    pub x: Unit,
    pub y: Unit,
}

impl Coord {

    #[inline]
    pub fn new(x: Unit, y: Unit) -> Coord {
        Coord {x, y}
    }

    #[inline]
    pub fn init(api_coords: ApiCoords) -> Coord {
        Coord {
            x: api_coords.x as Unit,
            y: api_coords.y as Unit,
        }
    }

    pub fn bounded_by(self, a: Coord, b: Coord) -> bool {
        let x_max = max(a.x, b.x);
        let x_min = min(a.x, b.x);
        let y_max = max(a.y, b.y);
        let y_min = min(a.y, b.y);
        self.x >= x_min && self.x <= x_max &&
            self.y >= y_min && self.y <= y_max
    }
}

impl Add<Offset> for Coord {
    type Output = Coord;

    #[inline]
    fn add(self, rhs: Offset) -> Coord {
        Coord {
            x: self.x + rhs.dx,
            y: self.x + rhs.dy,
        }
    }
}

impl Sub<Offset> for Coord {
    type Output = Coord;

    #[inline]
    fn sub(self, rhs: Offset) -> Coord {
        Coord {
            x: self.x - rhs.dx,
            y: self.x - rhs.dy,
        }
    }
}

impl Sub for Coord {
    type Output = Offset;

    #[inline]
    fn sub(self, rhs: Coord) -> Offset {
        Offset::between(self, rhs)
    }
}

impl AddAssign<Offset> for Coord {
    #[inline]
    fn add_assign(&mut self, rhs: Offset) {
        self.x += rhs.dx;
        self.x += rhs.dy;
    }
}

impl SubAssign<Offset> for Coord {
    #[inline]
    fn sub_assign(&mut self, rhs: Offset) {
        self.x -= rhs.dx;
        self.x -= rhs.dy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord_and_offset() {
        let mut a = Coord {x: 1, y: 2};
        let b = Coord {x: 10, y: 0};
        assert_ne!(a, b);

        let ab = Offset::between(a, b);
        a += ab;
        assert_eq!(a, b);
    }

    #[test]
    fn test_coord_ops() {
        let a = Coord {x: 1, y: 2};
        let b = Coord {x: 10, y: 0};

        let ab = a - b;
        assert_eq!(ab.dx, 9);
        assert_eq!(ab.dy, -2);

        let c = a + ab;
        assert_eq!(c.x, 10);
        assert_eq!(c.y, 0);
    }

    #[test]
    fn test_bound() {
        assert!(Coord {x: 5, y: 5}.bounded_by(
            Coord {x: 0, y: 0},
            Coord {x: 10, y: 10}
        ));

        assert!(Coord {x: 5, y: 5}.bounded_by(
            Coord {x: 10, y: 10},
            Coord {x: 0, y: 0}
        ));

        assert!(!Coord {x: 5, y: 5}.bounded_by(
            Coord {x: 0, y: 4},
            Coord {x: 4, y: 6}
        ));
    }
}
