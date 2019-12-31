use super::coord::Coord;
use crate::api::ApiCoords;

// A path connects a series of coordinates with direction. If nodes are only
// kept where the path begins, ends, and changes direction, then this path represents
// an exact path between the start and end. If any consecutive pair of nodes has a non-linear
// offset, the path between them is an undefined shortest manhattan path. All paths have at least
// one node, in which case its length is 0 but it can still intersect.
#[derive(PartialEq, Debug)]
pub struct Path {
    //todo: do nodes need to be public?
    pub nodes: Vec<Coord>,
}

impl Path {

    //todo: this is wrong... work on it
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

    pub fn length(&self) -> u16 {
        self.nodes.windows(2).fold(0, |total, pair| {
            total + (pair[1] - pair[0]).manhattan_dist()
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(Path::new(&[]), Option::None);

        let mut path = Path::new(&[
            Coord::new(1, 0),
        ]).unwrap();

        assert_eq!(path.nodes, &[
            Coord::new(1, 0),
        ]);

        path = Path::new(&[
            Coord::new(0, 0),
            Coord::new(1, 0),
        ]).unwrap();

        assert_eq!(path.nodes, &[
            Coord::new(0, 0),
            Coord::new(1, 0),
        ]);

        path = Path::new(&[
            Coord::new(0, 0),
            Coord::new(1, 0),
            Coord::new(2, 0),
            Coord::new(2, 1),
            Coord::new(2, 2),
            Coord::new(3, 2),
            Coord::new(3, 2),
        ]).unwrap();

        assert_eq!(path.nodes, &[
            Coord::new(0, 0),
            Coord::new(2, 0),
            Coord::new(2, 2),
            Coord::new(3, 2),
        ])
    }

    #[test]
    fn test_start_end() {
        let mut path = Path::new(&[
            Coord::new(0, 0),
        ]).unwrap();

        assert_eq!(path.start(), Coord::new(0, 0));
        assert_eq!(path.end(), Coord::new(0, 0));

        path = Path::new(&[
            Coord::new(0, 0),
            Coord::new(4, 0),
            Coord::new(4, 10),
            Coord::new(4, 10),
        ]).unwrap();

        assert_eq!(path.start(), Coord::new(0, 0));
        assert_eq!(path.end(), Coord::new(4, 10));
    }

    #[test]
    fn test_length() {
        assert_eq!(Path::new(&[
            Coord::new(0, 0),
        ]).unwrap().length(), 0);

        assert_eq!(Path::new(&[
            Coord::new(0, 0),
            Coord::new(1, 0),
        ]).unwrap().length(), 1);

        assert_eq!(Path::new(&[
            Coord::new(0, 0),
            Coord::new(2, 0),
        ]).unwrap().length(), 2);

        assert_eq!(Path::new(&[
            Coord::new(0, 0),
            Coord::new(1, 1),
        ]).unwrap().length(), 2);

        assert_eq!(Path::new(&[
            Coord::new(0, 0),
            Coord::new(1, 0),
            Coord::new(2, 0),
            Coord::new(2, 1),
            Coord::new(2, 2),
            Coord::new(3, 2),
            Coord::new(3, 2),
        ]).unwrap().length(), 5);

        assert_eq!(Path::new(&[
            Coord::new(0, 0),
            Coord::new(1, 0),
            Coord::new(1, 1),
            Coord::new(10, 10),
            Coord::new(15, 12),
        ]).unwrap().length(), 27);
    }

    #[test]
    fn test_intersects() {
        let mut path = Path::new(&[
            Coord::new(0, 0),
        ]).unwrap();

        assert!(path.intersects(Coord::new(0, 0)));
        assert!(!path.intersects(Coord::new(1, 0)));

        path = Path::new(&[
            Coord::new(0, 0),
            Coord::new(10, 0),
            Coord::new(10, 10),
            Coord::new(20, 20),
        ]).unwrap();

        assert!(path.intersects(Coord::new(5, 0)));
        assert!(!path.intersects(Coord::new(5, 1)));
        assert!(path.intersects(Coord::new(10, 5)));
        assert!(path.intersects(Coord::new(10, 10)));
        assert!(path.intersects(Coord::new(15, 15)));
        assert!(path.intersects(Coord::new(20, 20)));
        assert!(!path.intersects(Coord::new(20, 21)));
    }
}
