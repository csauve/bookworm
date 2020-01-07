use crate::game::*;
use std::time::{SystemTime, Duration};

//todo: how can I import my path! macro here?
pub fn run_benchmark() {
    println!("PATH");
    let mut path = Path::new(&[
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

    let path_side = 10;
    let mut times = 100_000;

    let start = SystemTime::now();
    while times > 0 {
        path.slide_start(Offset::new(0, path_side));
        path.slide_start(Offset::new(path_side, 0));
        path.slide_start(Offset::new(0, -path_side));
        path.slide_start(Offset::new(-path_side, 0));
        times -= 1;
    }

    let duration = SystemTime::now().duration_since(start).unwrap();
    println!("loop: {}ms", duration.as_millis());
}
