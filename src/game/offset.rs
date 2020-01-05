use std::ops::{Add, Sub, AddAssign, SubAssign};
use super::{Coord, Unit, UnitAbs};

pub const ZERO: Offset = Offset {
    dx: 0,
    dy: 0,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Offset {
    pub dx: Unit,
    pub dy: Unit,
}

impl Offset {

    #[inline]
    pub fn new(dx: Unit, dy: Unit) -> Offset {
        Offset {dx, dy}
    }

    #[inline]
    pub fn between(a: Coord, b: Coord) -> Offset {
        Offset {
            dx: b.x - a.x,
            dy: b.y - a.y,
        }
    }

    #[inline]
    pub fn linear(self) -> bool {
        (self.dx == 0) ^ (self.dy == 0)
    }

    #[inline]
    pub fn manhattan_dist(self) -> UnitAbs {
        self.dx.abs() as UnitAbs + self.dy.abs() as UnitAbs
    }
}

impl AddAssign for Offset {
    #[inline]
    fn add_assign(&mut self, rhs: Offset) {
        self.dx += rhs.dx;
        self.dy += rhs.dy;
    }
}

impl SubAssign for Offset {
    #[inline]
    fn sub_assign(&mut self, rhs: Offset) {
        self.dx -= rhs.dx;
        self.dy -= rhs.dy;
    }
}

impl Add for Offset {
    type Output = Offset;

    #[inline]
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

        let mut c = Offset::new(1, 1);
        c += Offset::new(2, -1);
        assert_eq!(c, Offset::new(2, 0))
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
