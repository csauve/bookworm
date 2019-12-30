use super::Offset;
use std::ops::{Add, Sub};
use std::cmp::{max, min};

//todo: split up this module

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coord {
    pub x: u8,
    pub y: u8,
}

impl Coord {
    pub fn translate(&self, offset: Offset) -> Coord {
        Coord {
            x: (self.x as i16 + offset.dx) as u8,
            y: (self.y as i16 + offset.dy) as u8,
        }
    }

    pub fn bounded_by(&self, a: Coord, b: Coord) -> bool {
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

    fn add(self, rhs: Offset) -> Coord {
        self.translate(rhs)
    }
}

impl Sub for Coord {
    type Output = Offset;

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
        assert_eq!(ab.dx, 9);
        assert_eq!(ab.dy, -2);
        assert_eq!(ab.manhattan_dist(), 11);

        let c = a.translate(ab);
        assert_eq!(c.x, 10);
        assert_eq!(c.y, 0);

        assert_eq!(c, b);
        assert_ne!(a, b);
    }
}
