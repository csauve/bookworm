use super::{Offset, Coord, UnitAbs};
use crate::api::{ApiCoords};

// A path connects a series of coordinates with direction.
// Nodes are only retained where the path has a discontinuity
// (start, end, change in direction). If any consecutive pair of
// nodes has a non-linear offset, the path between them is an
// undefined shortest manhattan path. All paths have at least
// one node, in which case its length is 0 but it can still intersect.

//todo: support stacked/overlapping paths with two distance fns
#[derive(PartialEq, Debug)]
pub struct Path {
    //todo: do nodes need to be public?
    pub nodes: Vec<Coord>,
}

impl Path {

    //todo: fix this so stacked coords are included
    pub fn new(arg_nodes: &[Coord]) -> Option<Path> {
        if arg_nodes.is_empty() {
            Option::None
        } else if arg_nodes.len() < 3 {
            Option::from(Path {nodes: Vec::from(arg_nodes)})
        } else {
            let mut nodes: Vec<Coord> = Vec::from(&arg_nodes[0..2]);

            for &next in &arg_nodes[2..] {
                let nodes_len = nodes.len();
                let prev_prev = nodes[nodes_len - 2];
                let prev = nodes[nodes_len - 1];
                if prev.bounded_by(prev_prev, next) && (next - prev_prev).linear() {
                    nodes[nodes_len - 1] = next;
                } else {
                    nodes.push(next);
                }
            }

            Option::from(Path {nodes})
        }
    }

    pub fn init(coords: &[ApiCoords]) -> Option<Path> {
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

    //todo: is the O(n) insert gonna be bad for performance when snakes move?
    pub fn extend_start(&mut self, offset: Offset) {
        let curr_start = self.nodes.first().unwrap();
        let new_start = *curr_start + offset;
        let is_continuous = self.nodes.len() >= 2 &&
            new_start != *curr_start &&
            offset.linear() &&
            curr_start.bounded_by(self.nodes[1], new_start);
        if is_continuous {
            *curr_start += offset;
        } else {
            self.nodes.insert(0, new_start);
        }
    }

    pub fn extend_end(&mut self, offset: Offset) {
        let curr_end = self.nodes.first().unwrap();
        let new_end = *curr_end + offset;
        let is_continuous = self.nodes.len() >= 2 &&
            new_end != *curr_end &&
            offset.linear() &&
            curr_end.bounded_by(self.nodes[self.nodes.len() - 2], new_end);
        if is_continuous {
            *curr_end += offset;
        } else {
            self.nodes.push(new_end);
        }
    }

    //todo
    pub fn trim_start(&mut self, dist: UnitAbs) {
        if dist == 1 {
            //todo: simple case
        } else if dist > 1 {
            //todo: complex case?
        }
    }

    //todo
    pub fn trim_end(&mut self, dist: UnitAbs) {
    }

    pub fn length(&self) -> UnitAbs {
        self.nodes.windows(2).fold(0, |total, pair| {
            total + (pair[1] - pair[0]).manhattan_dist()
        })
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    //todo: would this work without Coord implementing Copy/Clone?
    pub fn start(&self) -> Coord {
        *self.nodes.first().unwrap()
    }

    pub fn end(&self) -> Coord {
        *self.nodes.last().unwrap()
    }

    pub fn intersects(&self, coord: Coord) -> bool {
        //check the start and end first since they're likely to be common intersection points
        self.start() == coord || self.end() == coord || self.nodes.windows(2).any(|pair| {
            coord.bounded_by(pair[0], pair[1])
        })
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
    fn test_new() {
        assert_eq!(path![], Option::None);

        let mut path = path![
            (1, 0),
        ].unwrap();

        assert_eq!(path.nodes, &[
            Coord::new(1, 0),
        ]);

        path = path![
            (0, 0),
            (1, 0),
        ].unwrap();

        assert_eq!(path.nodes, &[
            Coord::new(0, 0),
            Coord::new(1, 0),
        ]);

        path = path![
            (0, 0),
            (1, 0),
            (2, 0),
            (2, 1),
            (2, 2),
            (3, 2),
            (3, 2),
        ].unwrap();

        assert_eq!(path.nodes, &[
            Coord::new(0, 0),
            Coord::new(2, 0),
            Coord::new(2, 2),
            Coord::new(3, 2),
        ])
    }

    #[test]
    fn test_start_end() {
        let mut path = path![
            (0, 0),
        ].unwrap();

        assert_eq!(path.start(), Coord::new(0, 0));
        assert_eq!(path.end(), Coord::new(0, 0));

        path = path![
            (0, 0),
            (4, 0),
            (4, 10),
            (4, 10),
        ].unwrap();

        assert_eq!(path.start(), Coord::new(0, 0));
        assert_eq!(path.end(), Coord::new(4, 10));
    }

    #[test]
    fn test_length() {
        assert_eq!(path![
            (0, 0),
        ].unwrap().length(), 0);

        assert_eq!(path![
            (0, 0),
            (1, 0),
        ].unwrap().length(), 1);

        assert_eq!(path![
            (0, 0),
            (2, 0),
        ].unwrap().length(), 2);

        assert_eq!(path![
            (0, 0),
            (1, 1),
        ].unwrap().length(), 2);

        assert_eq!(path![
            (0, 0),
            (1, 0),
            (2, 0),
            (2, 1),
            (2, 2),
            (3, 2),
            (3, 2),
        ].unwrap().length(), 5);

        assert_eq!(path![
            (0, 0),
            (1, 0),
            (1, 1),
            (10, 10),
            (15, 12),
        ].unwrap().length(), 27);
    }

    #[test]
    fn test_intersects() {
        let mut path = path![
            (0, 0),
        ].unwrap();

        assert!(path.intersects(Coord::new(0, 0)));
        assert!(!path.intersects(Coord::new(1, 0)));

        path = path![
            (0, 0),
            (10, 0),
            (10, 10),
            (20, 20),
        ].unwrap();

        assert!(path.intersects(Coord::new(5, 0)));
        assert!(!path.intersects(Coord::new(5, 1)));
        assert!(path.intersects(Coord::new(10, 5)));
        assert!(path.intersects(Coord::new(10, 10)));
        assert!(path.intersects(Coord::new(15, 15)));
        assert!(path.intersects(Coord::new(20, 20)));
        assert!(!path.intersects(Coord::new(20, 21)));
    }
}
