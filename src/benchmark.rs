use crate::game::path::Path;
use crate::game::coord::Coord;
use crate::game::offset::Offset;
use crate::game::get_decision;
use crate::api::{ApiDirection::*, ApiGameState};
use std::time::{SystemTime};
use log::*;

macro_rules! timed {
    ($name:expr, $code:block) => ({
        let start = SystemTime::now();
        $code
        let duration = SystemTime::now().duration_since(start).unwrap();
        info!("{}: {}ms", $name, duration.as_millis());
    });
}

pub fn run_benchmark() {
    path_slide();
    decision();
}

fn decision() {
    let game_state = ApiGameState::parse_basic("
    |  |  |  |  |  |  |  |  |  |  |()|  |
    |  |B2|B1|  |  |C2|C1|  |  |D1|D2|  |
    |  |  |B0|  |  |  |C0|  |  |D0|  |  |
    |  |()|  |  |  |  |  |  |  |  |  |  |
    |  |  |  |  |  |  |  |  |()|  |  |  |
    |  |A1|A0|  |  |  |()|  |  |E0|E1|  |
    |  |A2|  |  |  |  |  |  |  |  |E2|  |
    |  |  |  |  |  |  |  |  |()|  |  |  |
    |  |  |  |  |()|  |  |  |  |  |  |  |
    |  |  |Y0|  |  |  |G0|  |  |F0|F1|  |
    |  |Y2|Y1|  |  |  |G1|G2|  |  |F2|  |
    |  |  |  |()|  |  |  |  |  |  |  |  |
    ");

    timed!("get_decision x1", {
        for _ in 0..1 {
            get_decision(&game_state);
        }
    });
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

    timed!("path_slide x100_000", {
        for i in 0..100_000 {
            path.slide_start(Offset::from(moves[i % moves.len()]));
        }
    });
}
