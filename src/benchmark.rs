use std::str;
use std::time::{SystemTime};
use log::*;
use crate::game::{Path, Coord, Offset, Board};
use crate::brain::get_decision;
use crate::api::{ApiDirection::*, ApiGameState};

macro_rules! timed {
    ($name:expr, $code:block) => ({
        let start = SystemTime::now();
        $code
        let duration = SystemTime::now().duration_since(start).unwrap().as_nanos();
        info!("{}: {} ns", $name, fmt_int(duration));
    });
    ($name:expr, $n:expr, $code:expr) => ({
        let start = SystemTime::now();
        for i in 0..$n {
            let _ = $code(i);
        }
        let duration = SystemTime::now().duration_since(start).unwrap().as_nanos() / $n;
        info!("{}: {} ns (avg. of {})", $name, fmt_int(duration), fmt_int($n as u128));
    });
}

fn fmt_int(integer: u128) -> String {
    String::from_utf8(integer
        .to_string()
        .bytes()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(",")
        .bytes()
        .rev()
        .collect::<Vec<_>>()
    ).unwrap()
}

pub fn run_benchmark() {
    path_slide();
    board();
    decision();
}

fn board() {
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
    let board = Board::from_api(&game_state);

    timed!("get_free_moves", 1_000, |_| {
        board.enumerate_snake_moves()
    });

    timed!("pathfind", 1_000, |_| {
        let _path = board.pathfind(board.you().head(), Coord::new(11, 11));
    });

    timed!("territories", 10_000, |_| {
        let _territories = board.get_territories();
    });
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

    timed!("get_decision_early_game", 3, |_| {
        get_decision(&game_state);
    });

    let game_state = ApiGameState::parse_basic("
    |   |   |   |   |A17|   |   |   |   |   |() |
    |   |   |   |   |A16|   |   |   |   |   |   |
    |   |   |   |   |A15|A14|A13|A12|   |   |   |
    |Y6 |Y5 |Y4 |Y3 |   |   |   |A11|   |   |   |
    |Y7 |   |   |Y2 |   |   |   |A10|   |   |   |
    |Y8 |   |   |Y1 |   |   |   |A9 |   |   |   |
    |Y9 |Y10|Y11|Y0 |   |   |A7 |A8 |   |   |   |
    |   |   |   |   |A0 |   |A6 |   |   |   |   |
    |   |   |   |   |A1 |   |A5 |   |   |   |   |
    |   |   |   |   |A2 |A3 |A4 |   |   |   |   |
    |   |() |   |   |   |   |   |   |   |   |() |
    ");

    timed!("get_decision_late_game", 3, |_| {
        get_decision(&game_state);
    });

    //this is a particularly bad case for choosing priority snakes
    let game_state = ApiGameState::parse_basic("
    |  |  |  |  |  |  |  |  |  |  |  |
    |  |  |  |  |  |  |  |  |  |  |  |
    |  |  |  |  |  |  |  |  |  |  |  |
    |  |  |  |  |  |  |  |  |  |  |  |
    |  |  |  |  |  |  |D0|D1|D2|  |  |
    |  |  |  |B1|B0|  |  |  |  |  |  |
    |  |  |  |C1|C0|Y0|  |  |  |  |  |
    |  |  |  |  |  |Y1|A0|A1|A2|  |  |
    |  |  |  |  |  |Y2|  |  |  |  |  |
    |  |  |  |  |  |  |  |  |  |  |  |
    |  |  |  |  |  |  |  |  |  |  |  |
    ");

    timed!("get_decision_tricky", {
        get_decision(&game_state);
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

    timed!("path_slide", 100_000, |i| {
        path.slide_start(Offset::from(moves[i % moves.len()]));
    });
}
