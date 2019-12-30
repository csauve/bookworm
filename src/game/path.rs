use super::coord::Coord;
use super::offset;
use crate::api::ApiCoords;

// A path connects a series of coordinates with direction. Nodes are only
// kept where the path begins, ends, and changes direction.
#[derive(PartialEq)]
pub struct Path {
    //todo: do nodes need to be public?
    pub nodes: Vec<Coord>,
}

impl Path {

    //todo: should we allow 0-length paths?
    //todo: is this backwards for snakes?
    pub fn init(coords: &Vec<ApiCoords>) -> Path {
        let mut nodes: Vec<Coord> = Vec::new();
        let mut prev_offset = offset::ZERO;

        //todo: convert ApiCoords to Path
        for coord in coords.iter() {
            let next = Coord {
                //todo: handle overflows?
                x: coord.x as u8,
                y: coord.y as u8,
            };
            if let Some(prev) = nodes.last() {
                //todo: use and update prev_offset instead
                if next == *prev {
                    //stacked ApiCoords can be skipped
                    continue;
                }
            } else {
                //it's the first node, so add it regardless
                nodes.push(next);
            }
        }

        Path {nodes}
    }

    //todo: this isn't correct, work on it and add tests
    pub fn length(&self) -> u16 {
        self.nodes.windows(2).fold(1, |total, pair| {
            total + (pair[1] - pair[0]).manhattan_dist()
        })
    }

    //todo: would does this work without Coord implementing Copy/Clone?
    pub fn start(&self) -> Coord {
        *self.nodes.first().unwrap()
    }

    pub fn end(&self) -> Coord {
        *self.nodes.last().unwrap()
    }

    pub fn intersects(&self, coord: Coord) -> bool {
        self.start() == coord || self.nodes.windows(2).any(|pair| {
            coord.bounded_by(pair[0], pair[1])
        })
    }
}

//todo: tests for Path
