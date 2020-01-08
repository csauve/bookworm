use crate::game::*;
use crate::api::ApiDirection::*;
use std::time::{SystemTime};

macro_rules! timed {
    ($name:expr, $code:block) => ({
        let start = SystemTime::now();
        $code
        let duration = SystemTime::now().duration_since(start).unwrap();
        println!("{}: {}ms", $name, duration.as_millis());
    });
}

//todo: how can I import my path! macro here?
pub fn run_benchmark() {
    path_slide();
}

fn path_slide() {
    let mut path = Path::from_slice(&[
        Coord::new(0, 0),
        Coord::new(1, 0),
        Coord::new(2, 0),
        Coord::new(3, 0),
        Coord::new(4, 0),
        Coord::new(5, 0),
        Coord::new(6, 0),
        Coord::new(7, 0),
        Coord::new(8, 0),
        Coord::new(9, 0),
    ]);

    let moves = &[Down, Right, Down, Left, Down, Right, Right, Right, Up, Left, Up, Right, Up, Left, Left, Left];

    timed!("path_slide", {
        for i in 0..100_000 {
            path.slide_start(Offset::from(moves[i % moves.len()]));
        }
    });
}
