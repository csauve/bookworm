use rayon::prelude::*;
use log::*;
use hyper::{Client, Request, Body};
use crate::game::turn::Turn;
use crate::game::coord::UnitAbs;
use crate::api::*;

pub async fn run_game(_timeout_ms: u32, snakes_addrs: Vec<String>, width: UnitAbs, height: UnitAbs) {
    info!("Initializing {}x{} board", width, height);
    let num_snakes = snakes_addrs.len();
    let mut turn = Turn::init(width, height, num_snakes).unwrap();

    //todo: set timeout
    let mut client = Client::default();

    let snake_configs = snakes_addrs.par_iter().map(|addr| async {
        client.call(Request::post(format!("{}/start", addr))
            .body(Body::from("todo"))
            .unwrap()
        ).await
    }).collect::<Vec<_>>();

    while turn.snakes.len() > 1 {
        let snake_moves = snakes_addrs.par_iter().map(|addr| async {
            let game_state = build_api_game_state(&turn);
            client.call(Request::post(format!("{}/move", addr))
                .body(Body::from(serde_json::to_string(&game_state).unwrap()))
                .unwrap()
            ).await
        }).collect::<Vec<_>>().await;

        let deaths = turn.advance(&snake_moves);

        //notify dead snakes about /end
    }

    //notify winner (may be none if both died in final turn)
}

fn build_api_game_state(turn: &Turn) -> ApiGameState {
    ApiGameState {
        //...
    }
}
