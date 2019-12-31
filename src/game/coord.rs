use crate::api::ApiCoords;
use super::Offset;
use std::ops::{Add, Sub};
use std::cmp::{max, min};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coord {
    pub x: u8,
    pub y: u8,
}

impl Coord {

    #[inline]
    pub fn new(x: u8, y: u8) -> Coord {
        Coord {x, y}
    }

    #[inline]
    pub fn init(api_coords: ApiCoords) -> Coord {
        //todo: handle overflows?
        Coord {
            x: api_coords.x as u8,
            y: api_coords.y as u8,
        }
    }

    #[inline]
    pub fn translate(self, offset: Offset) -> Coord {
        Coord {
            x: (self.x as i16 + offset.dx) as u8,
            y: (self.y as i16 + offset.dy) as u8,
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
        self.translate(rhs)
    }
}

impl Sub for Coord {
    type Output = Offset;

    #[inline]
    fn sub(self, rhs: Coord) -> Offset {
        Offset::between(self, rhs)
    }
}

//todo: split up tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord_and_offset() {
        let a = Coord {x: 1, y: 2};
        let b = Coord {x: 10, y: 0};

        let ab = Offset::between(a, b);
        let c = a.translate(ab);
        assert_eq!(c.x, 10);
        assert_eq!(c.y, 0);

        assert_eq!(c, b);
        assert_ne!(a, b);
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
