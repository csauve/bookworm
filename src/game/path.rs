use crate::api::{ApiCoords};
use super::coord::{Coord, UnitAbs};
use super::offset::{Offset};

// A path connects a series of coordinates with direction.
// If any consecutive pair of nodes has a non-linear offset,
// the path between them is an undefined shortest manhattan path.
#[derive(Clone, PartialEq, Debug)]
pub struct Path {
    nodes: Vec<Coord>,
}

impl Path {

    pub fn new() -> Path {
        Path {nodes: Vec::new()}
    }

    pub fn from_slice(nodes: &[Coord]) -> Path {
        Path {nodes: Vec::from(nodes)}
    }

    pub fn from_vec(nodes: Vec<Coord>) -> Path {
        Path {nodes}
    }

    pub fn from_api(coords: &[ApiCoords]) -> Path {
        let mapped_coords: Vec<_> = coords.iter().map(Coord::from).collect();
        Path::from_vec(mapped_coords)
    }

    pub fn slide_start(&mut self, offset: Offset) {
        self.extend_start(offset);
        self.pop_end();
    }

    pub fn slide_end(&mut self, offset: Offset) {
        self.extend_end(offset);
        self.pop_start();
    }

    pub fn extend_start(&mut self, offset: Offset) {
        if !self.nodes.is_empty() {
            let curr_start = self.nodes.first().unwrap();
            let new_start = *curr_start + offset;
            self.nodes.insert(0, new_start);
        }
    }

    pub fn extend_end(&mut self, offset: Offset) {
        if !self.nodes.is_empty() {
            let curr_end = self.nodes.last().unwrap();
            let new_end = *curr_end + offset;
            self.nodes.push(new_end);
        }
    }

    pub fn pop_start(&mut self) -> Option<Coord> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.nodes.remove(0))
        }
    }

    pub fn pop_end(&mut self) -> Option<Coord> {
        self.nodes.pop()
    }

    pub fn dist(&self) -> UnitAbs {
        self.nodes.windows(2).fold(0, |total, pair| {
            total + (pair[1] - pair[0]).manhattan_dist()
        })
    }

    //each node, and the space between them, is to be considered a step
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn start(&self) -> Option<Coord> {
        self.nodes.first().cloned()
    }

    pub fn end(&self) -> Option<Coord> {
        self.nodes.last().cloned()
    }

    pub fn get_node(&self, index: usize) -> Option<Coord> {
        self.nodes.get(index).cloned()
    }

    //todo: maybe won't need this code
    pub fn intersects(&self, coord: Coord) -> bool {
        !self.nodes.is_empty() && (
            //check the start and end first since they're likely to be common intersection points
            self.start().unwrap() == coord ||
            self.end().unwrap() == coord ||
            self.nodes.windows(2).any(|pair| {coord.bounded_by(pair[0], pair[1])})
        )
    }

    pub fn contains_node(&self, coord: Coord) -> bool {
        self.nodes.contains(&coord)
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
    ($(($x:expr, $y:expr)),*) => (Path::from_slice(&[$(Coord::new($x, $y)),*]));
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

        let path = path![
            (1, 0),
        ];

        assert_eq!(path.nodes, &[
            Coord::new(1, 0),
        ]);

        let path = path![
            (0, 0),
            (1, 0),
        ];

        assert_eq!(path.nodes, &[
            Coord::new(0, 0),
            Coord::new(1, 0),
        ]);
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
        //dist, num_nodes
        macro_rules! check_len {
            ($dist:expr, $num_nodes:expr, $path:expr) => ({
                assert_eq!($path.dist(), $dist);
                assert_eq!($path.num_nodes(), $num_nodes);
            });
        }

        check_len!(0, 0, path![]);
        check_len!(0, 1, path![(0, 0)]);

        check_len!(1, 2, path![
            (0, 0),
            (1, 0),
        ]);

        check_len!(2, 2, path![
            (0, 0),
            (2, 0),
        ]);

        check_len!(2, 2, path![
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

        check_len!(27, 5, path![
            (0, 0),
            (1, 0),
            (1, 1),
            (10, 10),
            (15, 12),
        ]);

        check_len!(10, 13, path![
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
    fn test_extend() {
        let mut path = path![];
        path.extend_end(Offset::new(0, 1));
        path.extend_start(Offset::new(0, 1));
        assert_eq!(path.num_nodes(), 0);

        let mut path = path![(1, 1)];
        assert_eq!(path.end().unwrap(), Coord::new(1, 1));
        assert_eq!(path.num_nodes(), 1);
        path.extend_end(Offset::new(0, 2));
        assert_eq!(path.start().unwrap(), Coord::new(1, 1));
        assert_eq!(path.end().unwrap(), Coord::new(1, 3));
        assert_eq!(path.num_nodes(), 2);
        path.extend_start(Offset::new(-1, -1));
        assert_eq!(path.start().unwrap(), Coord::new(0, 0));
        assert_eq!(path.num_nodes(), 3);
    }

    #[test]
    fn test_pop() {
        let mut path = path![];
        assert_eq!(path.pop_end(), None);
        assert_eq!(path.pop_start(), None);

        let mut path = path![(1, 0), (2, 0), (3, 0)];
        assert_eq!(path.pop_end(), Some(Coord::new(3, 0)));
        assert_eq!(path.pop_start(), Some(Coord::new(1, 0)));
        assert_eq!(path.num_nodes(), 1);
    }

    #[test]
    fn test_slide() {
        let mut path = path![(1, 0), (2, 0), (3, 0)];
        path.slide_start(Offset::new(-1, 0));
        assert_eq!(path.start().unwrap(), Coord::new(0, 0));
        assert_eq!(path.end().unwrap(), Coord::new(2, 0));
        assert_eq!(path.num_nodes(), 3);

        path.slide_end(Offset::new(0, 1));
        assert_eq!(path.start().unwrap(), Coord::new(1, 0));
        assert_eq!(path.end().unwrap(), Coord::new(2, 1));
        assert_eq!(path.num_nodes(), 3);
    }
}
