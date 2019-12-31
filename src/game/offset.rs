use std::ops::{Add, Sub};
use super::Coord;

pub const ZERO: Offset = Offset {
    dx: 0,
    dy: 0,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Offset {
    pub dx: i16,
    pub dy: i16,
}

impl Offset {

    pub fn new(dx: i16, dy: i16) -> Offset {
        Offset {dx, dy}
    }

    pub fn between(a: Coord, b: Coord) -> Offset {
        Offset {
            dx: b.x as i16 - a.x as i16,
            dy: b.y as i16 - a.y as i16,
        }
    }

    #[inline]
    pub fn linear(self) -> bool {
        (self.dx == 0) ^ (self.dy == 0)
    }

    #[inline]
    pub fn manhattan_dist(self) -> u16 {
        self.dx.abs() as u16 + self.dy.abs() as u16
    }
}

//todo: implement AddAssign and SubAssign as well
impl Add for Offset {
    type Output = Offset;

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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_ops() {
        let a = Offset::new(4, -2);
        let b = Offset::new(-1, 10);

        assert_eq!(a + b, Offset::new(3, 8));
        assert_eq!(a - b, Offset::new(5, -12));
        assert_eq!(Offset::new(0, 0), ZERO);
    }

    #[test]
    fn test_linear() {
        assert!(Offset::new(0, 10).linear());
        assert!(Offset::new(-1, 0).linear());
        assert!(!Offset::new(-1, 1).linear());
        assert!(!Offset::new(0, 0).linear());
    }

    #[test]
    fn test_manhattan_dist() {
        assert_eq!(Offset::new(0, 10).manhattan_dist(), 10);
        assert_eq!(Offset::new(0, -10).manhattan_dist(), 10);
        assert_eq!(Offset::new(10, 0).manhattan_dist(), 10);
        assert_eq!(Offset::new(-10, 0).manhattan_dist(), 10);
        assert_eq!(Offset::new(-10, 10).manhattan_dist(), 20);
        assert_eq!(Offset::new(1, 2).manhattan_dist(), 3);
        assert_eq!(Offset::new(2, 1).manhattan_dist(), 3);
    }
}
