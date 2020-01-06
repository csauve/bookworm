use crate::api::{ApiCoords};
use super::{Offset, Coord, UnitAbs};
use std::cmp::max;

// A path connects a series of coordinates with direction.
// Nodes are only retained where the path has a discontinuity
// (start, end, change in direction). If any consecutive pair of
// nodes has a non-linear offset, the path between them is an
// undefined shortest manhattan path.
#[derive(PartialEq, Debug)]
pub struct Path {
    nodes: Vec<Coord>,
}

//todo: would this just be simpler as a non-optimizing list of nodes?
impl Path {

    pub fn new(arg_nodes: &[Coord]) -> Path {
        if arg_nodes.len() < 3 {
            Path {nodes: Vec::from(arg_nodes)}
        } else {
            let mut nodes: Vec<Coord> = Vec::from(&arg_nodes[0..2]);

            for &next in &arg_nodes[2..] {
                let nodes_len = nodes.len();
                let prev_prev = nodes[nodes_len - 2];
                let prev = nodes[nodes_len - 1];
                if Path::is_continuous(prev_prev, prev, next) {
                    nodes[nodes_len - 1] = next;
                } else {
                    nodes.push(next);
                }
            }

            Path {nodes}
        }
    }

    pub fn init(coords: &[ApiCoords]) -> Path {
        let mapped_coords: Vec<_> = coords.iter().map(|&c| Coord::init(c)).collect();
        Path::new(&mapped_coords)
    }

    pub fn slide_start(&mut self, offset: Offset) {
        self.extend_start(offset);
        self.trim_end(offset.manhattan_dist());
    }

    pub fn slide_end(&mut self, offset: Offset) {
        self.extend_end(offset);
        self.trim_start(offset.manhattan_dist());
    }

    #[inline]
    fn is_continuous(a: Coord, b: Coord, c: Coord) -> bool {
        b != a && b != c && a != c && b.bounded_by(a, c) && (c - a).linear()
    }

    pub fn extend_start(&mut self, offset: Offset) {
        if !self.nodes.is_empty() {
            let curr_start = self.nodes.first().unwrap();
            let new_start = *curr_start + offset;
            if self.nodes.len() >= 2 && Path::is_continuous(self.nodes[1], *curr_start, new_start) {
                *self.nodes.first_mut().unwrap() += offset;
            } else {
                self.nodes.insert(0, new_start);
            }
        }
    }

    pub fn extend_end(&mut self, offset: Offset) {
        if !self.nodes.is_empty() {
            let curr_end = self.nodes.last().unwrap();
            let new_end = *curr_end + offset;
            if self.nodes.len() >= 2 && Path::is_continuous(self.nodes[self.nodes.len() - 2], *curr_end, new_end) {
                *self.nodes.last_mut().unwrap() += offset;
            } else {
                self.nodes.push(new_end);
            }
        }
    }

    pub fn trim_start(&mut self, steps: UnitAbs) {
        let mut remaining_steps = steps;
        while self.nodes.len() > 1 {
            let edge = self.nodes[0];
            let next = self.nodes[1];
            let pair_offset = edge - next;
            let pair_steps = max(1, pair_offset.manhattan_dist());
            if remaining_steps < pair_steps {
                self.nodes[0].move_toward(next, remaining_steps);
                remaining_steps = 0;
                break;
            }
            self.nodes.remove(0);
            remaining_steps -= pair_steps;
        }
        if remaining_steps > 0 && !self.nodes.is_empty() {
            self.nodes.clear();
        }
    }

    pub fn trim_end(&mut self, steps: UnitAbs) {
        let mut remaining_steps = steps;
        while self.nodes.len() > 1 {
            let curr_len = self.nodes.len();
            let edge = self.nodes[curr_len - 1];
            let next = self.nodes[curr_len - 2];
            let pair_offset = edge - next;
            let pair_steps = max(1, pair_offset.manhattan_dist());
            if remaining_steps < pair_steps {
                self.nodes[curr_len - 1].move_toward(next, remaining_steps);
                remaining_steps = 0;
                break;
            }
            self.nodes.pop();
            remaining_steps -= pair_steps;
        }
        if remaining_steps > 0 && !self.nodes.is_empty() {
            self.nodes.clear();
        }
    }

    pub fn dist(&self) -> UnitAbs {
        self.nodes.windows(2).fold(0, |total, pair| {
            total + (pair[1] - pair[0]).manhattan_dist()
        })
    }

    //each node, and the space between them, is to be considered a step
    pub fn steps(&self) -> UnitAbs {
        if self.nodes.len() < 2 {
            self.nodes.len() as UnitAbs
        } else {
            self.nodes.windows(2).fold(1, |total, pair| {
                total + max(1, (pair[1] - pair[0]).manhattan_dist())
            })
        }
    }

    pub fn start(&self) -> Option<Coord> {
        self.nodes.first().cloned()
    }

    pub fn end(&self) -> Option<Coord> {
        self.nodes.last().cloned()
    }

    pub fn intersects(&self, coord: Coord) -> bool {
        !self.nodes.is_empty() && (
            //check the start and end first since they're likely to be common intersection points
            self.start().unwrap() == coord ||
            self.end().unwrap() == coord ||
            self.nodes.windows(2).any(|pair| {coord.bounded_by(pair[0], pair[1])})
        )
    }

    pub fn start_self_intersects(&self) -> bool {
        if let Some(start) = self.nodes.first() {
            self.nodes.windows(2).skip(1).any(|pair| {start.bounded_by(pair[0], pair[1])})
        } else {
            false
        }
    }
}

#[macro_export]
macro_rules! path {
    ($(($x:expr, $y:expr)),*) => (Path::new(&[$(Coord::new($x, $y)),*]));
    ($($coord:expr),*) => (Path::new(&[$($coord),*]));
    ($(($x:expr, $y:expr),)*) => (path![$(($x, $y)),*]);
    ($($coord:expr,)*) => (path![$($coord),*]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_basic() {
        let empty = path![];
        assert_eq!(empty.nodes, &[]);
        assert_eq!(empty.start(), None);
        assert_eq!(empty.end(), None);

        let mut path = path![
            (1, 0),
        ];

        assert_eq!(path.nodes, &[
            Coord::new(1, 0),
        ]);

        path = path![
            (0, 0),
            (1, 0),
        ];

        assert_eq!(path.nodes, &[
            Coord::new(0, 0),
            Coord::new(1, 0),
        ]);
    }

    #[test]
    fn test_new_simplify() {
        let path = path![
            (0, 0),
            (1, 0),
            (2, 0), //corner
            (2, 1),
            (2, 2), //corner
            (3, 2), //stacked
            (3, 2),
            (4, 2),
            (5, 2), //corner
            (5, 3),
            (5, 4),
            (5, 5), //stacked
            (5, 5),
        ];

        assert_eq!(path.nodes, &[
            Coord::new(0, 0),
            Coord::new(2, 0),
            Coord::new(2, 2),
            Coord::new(3, 2),
            Coord::new(3, 2),
            Coord::new(5, 2),
            Coord::new(5, 5),
            Coord::new(5, 5),
        ])
    }

    #[test]
    fn test_start_end() {
        let mut path = path![
            (0, 0),
        ];

        assert_eq!(path.start().unwrap(), Coord::new(0, 0));
        assert_eq!(path.end().unwrap(), Coord::new(0, 0));

        path = path![
            (0, 0),
            (4, 0),
            (4, 10),
            (4, 10),
        ];

        assert_eq!(path.start().unwrap(), Coord::new(0, 0));
        assert_eq!(path.end().unwrap(), Coord::new(4, 10));
    }

    //todo: len vs dist
    #[test]
    fn test_length() {
        //dist, steps
        macro_rules! check_len {
            ($dist:expr, $steps:expr, $path:expr) => ({
                assert_eq!($path.steps(), $steps);
                assert_eq!($path.dist(), $dist);
            });
        }

        check_len!(0, 0, path![]);
        check_len!(0, 1, path![(0, 0)]);

        check_len!(1, 2, path![
            (0, 0),
            (1, 0),
        ]);

        check_len!(2, 3, path![
            (0, 0),
            (2, 0),
        ]);

        check_len!(2, 3, path![
            (0, 0),
            (1, 1),
        ]);

        check_len!(5, 7, path![
            (0, 0),
            (1, 0),
            (2, 0),
            (2, 1),
            (2, 2),
            (3, 2),
            (3, 2),
        ]);

        //28?
        check_len!(27, 28, path![
            (0, 0),
            (1, 0),
            (1, 1),
            (10, 10),
            (15, 12),
        ]);
    }

    #[test]
    fn test_intersects() {
        let mut path = path![
            (0, 0),
        ];

        assert!(path.intersects(Coord::new(0, 0)));
        assert!(!path.intersects(Coord::new(1, 0)));

        let path = path![
            (0, 0),
            (10, 0),
            (10, 10),
            (20, 20),
        ];

        assert!(path.intersects(Coord::new(5, 0)));
        assert!(!path.intersects(Coord::new(5, 1)));
        assert!(path.intersects(Coord::new(10, 5)));
        assert!(path.intersects(Coord::new(10, 10)));
        assert!(path.intersects(Coord::new(15, 15)));
        assert!(path.intersects(Coord::new(20, 20)));
        assert!(!path.intersects(Coord::new(20, 21)));
    }

    #[test]
    fn test_trim_edge_cases() {
        let mut path = path![];
        assert_eq!(path.steps(), 0);
        assert_eq!(path.dist(), 0);
        path.trim_end(3);
        path.trim_start(3);
        assert_eq!(path.nodes, &[]);
        assert_eq!(path.steps(), 0);
        assert_eq!(path.dist(), 0);

        let mut path = path![(1, 1)];
        assert_eq!(path.steps(), 1);
        assert_eq!(path.dist(), 0);
        assert_eq!(path.end().unwrap(), Coord::new(1, 1));
        path.trim_end(0);
        assert_eq!(path.steps(), 1);
        path.trim_end(3);
        assert_eq!(path.nodes, &[]);
        assert_eq!(path.steps(), 0);
        assert_eq!(path.end(), None);
    }

    #[test]
    fn test_trim_continuous() {
        let mut path = path![
            (1, 1),
            (2, 1),
            (3, 1),
            (3, 2),
        ];
        assert_eq!(path.steps(), 4);
        assert_eq!(path.dist(), 3);
        assert_eq!(path.start().unwrap(), Coord::new(1, 1));
        path.trim_start(1);
        assert_eq!(path.start().unwrap(), Coord::new(2, 1)); //fix me!!!
        assert_eq!(path.steps(), 3);
        assert_eq!(path.dist(), 2);
        path.trim_end(2);
        assert_eq!(path.start().unwrap(), Coord::new(2, 1));
        assert_eq!(path.end().unwrap(), Coord::new(2, 1));
        assert_eq!(path.steps(), 1);
        assert_eq!(path.dist(), 0);
    }

    #[test]
    fn test_trim_discontinuous() {
        //snakes are like this at start of game
        let mut path = path![
            (2, 1),
            (2, 1),
            (2, 1),
        ];
        assert_eq!(path.nodes.len(), 3);
        assert_eq!(path.steps(), 3);
        assert_eq!(path.dist(), 0);
        path.trim_end(1);
        assert_eq!(path.nodes.len(), 2);
        assert_eq!(path.steps(), 2);
        assert_eq!(path.dist(), 0);

        let mut path = path![
            (1, 1),
            (1, 1), //4
            (2, 1),
            (2, 2),
            (2, 2),
            (3, 2),
            (4, 2),
        ];
        assert_eq!(path.steps(), 7);
        assert_eq!(path.dist(), 4);
        path.trim_start(1);
        assert_eq!(path.steps(), 6);
        assert_eq!(path.dist(), 4);
        assert_eq!(path.start().unwrap(), Coord::new(1, 1));
        path.trim_start(4);
        assert_eq!(path.steps(), 2);
        assert_eq!(path.dist(), 1);
        assert_eq!(path.start().unwrap(), Coord::new(3, 2));
    }

    //todo: test slide; len() is maintained, but dist may not be
    #[test]
    fn test_slide() {
        let mut path = path![];
        assert_eq!(path.steps(), 0);
        path.slide_start(Offset::new(1, 0));
        path.slide_end(Offset::new(1, 0));
        assert_eq!(path.steps(), 0);

        let mut path = path![(1, 0)];
        assert_eq!(path.steps(), 1);
        path.slide_start(Offset::new(1, 0));
        path.slide_end(Offset::new(1, 0));
        assert_eq!(path.start().unwrap(), Coord::new(3, 0));

        let mut path = path![
            (2, 1),
            (2, 1),
            (2, 1),
        ];
        assert_eq!(path.nodes.len(), 3);
        assert_eq!(path.steps(), 3);
        path.slide_start(Offset::new(1, 0));
        path.slide_start(Offset::new(0, 1));
        assert_eq!(path.start().unwrap(), Coord::new(3, 2));
        assert_eq!(path.end().unwrap(), Coord::new(2, 1));
        assert_eq!(path.steps(), 3);
        path.slide_start(Offset::new(0, 1));
        assert_eq!(path.end().unwrap(), Coord::new(3, 1));
        assert_eq!(path.start().unwrap(), Coord::new(3, 3));
        assert_eq!(path.steps(), 3);
        assert_eq!(path.nodes.len(), 2); //should have simplified the path since it's straight now

        path.slide_end(Offset::new(7, 0));
        assert_eq!(path.end().unwrap(), Coord::new(10, 1));
        assert_eq!(path.start().unwrap(), Coord::new(8, 1));
        assert_eq!(path.steps(), 3);

        path.slide_end(Offset::new(0, -1));
        assert_eq!(path.steps(), 3);
        assert_eq!(path.nodes.len(), 3); //should have inserted a new node
    }
}
