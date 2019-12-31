use super::coord::Coord;
use super::offset;
use crate::api::ApiCoords;

// A path connects a series of coordinates with direction. If nodes are only
// kept where the path begins, ends, and changes direction, then this path represents
// an exact path between the start and end. If any consecutive pair of nodes has a non-linear
// offset, the path between them is an undefined shortest manhattan path. All paths have at least
// one node, in which case its length is 0 but it can still intersect.
#[derive(PartialEq)]
pub struct Path {
    //todo: do nodes need to be public?
    pub nodes: Vec<Coord>,
}

impl Path {

    //todo: should we allow 0-length paths?
    pub fn new(arg_nodes: &[Coord]) -> Option<Path> {
        if arg_nodes.is_empty() {
            return Option::None;
        }

        let mut nodes: Vec<Coord> = Vec::new();
        let mut prev_offset = offset::ZERO;

        //todo: this is wrong... work on it
        for next in arg_nodes.iter() {
            if let Some(prev) = nodes.last() {
                let next_offset = *next - *prev;
                if next_offset != prev_offset && *next != *prev {
                    nodes.push(*next);
                }
                prev_offset = next_offset;
            } else {
                //it's the first node, so add it regardless
                nodes.push(*next);
            }
        }

        Option::from(Path {nodes})
    }

    //todo: convert to slice argument
    pub fn init(coords: &Vec<ApiCoords>) -> Option<Path> {
        let mapped_coords: Vec<_> = coords.iter().map(|c| Coord::init(*c)).collect();
        Path::new(&mapped_coords)
    }

    //todo: this isn't correct, work on it and add tests
    pub fn length(&self) -> u16 {
        self.nodes.windows(2).fold(1, |total, pair| {
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
        let path = Path::new(&[Coord::new(0, 0)]);
    }

    #[test]
    fn test_init() {
    }

}
