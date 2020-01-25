use std::sync::Mutex;
use futures::future;
use log::*;
use uuid::Uuid;
use hyper::{Client, Request, Body, body, client::connect::HttpConnector};
use crate::game::turn::Turn;
use crate::game::snake::Snake;
use crate::game::coord::UnitAbs;
use crate::api::*;

struct LiveSnake {
    pub id: ApiSnakeId,
    pub addr: String,
    pub config: ApiSnakeConfig,
}

async fn notify_start(client: &Client<HttpConnector>, addr: &str, game_state: ApiGameState) -> Result<ApiSnakeConfig, String> {
    let req = Request::post(format!("{}/start", addr))
        .body(Body::from(serde_json::to_string(&game_state).unwrap()))
        .unwrap();
    match client.request(req).await {
        Err(e) => {
            Err(format!("Snake @ {} failed to reply: {}", addr, e))
        },
        Ok(res) => {
            let res_body = res.into_body();
            match serde_json::from_slice::<ApiSnakeConfig>(&body::to_bytes(res_body).await.unwrap()) {
                Ok(snake_config) => Ok(snake_config),
                Err(e) => {
                    Err(format!("Snake @ {} responded with invalid JSON: {}", addr, e))
                }
            }
        }
    }
}

pub async fn run_game(_timeout_ms: u32, snakes_addrs: &[String], width: UnitAbs, height: UnitAbs) {
    info!("Initializing {}x{} board", width, height);
    let mut turn = Turn::init(width, height, snakes_addrs.len()).unwrap();
    let mut turn_index: u32 = 0;
    let game_id: ApiGameId = Uuid::new_v4().to_string();;
    let client = Client::default(); //todo: set timeout

    info!("Notifying snakes of game start; id: {}", &game_id);
    let start_responses = future::join_all(
        snakes_addrs.iter().enumerate().map(|(snake_index, addr)| {
            let game_state = build_api_game_state(&turn, snake_index, turn_index, &game_id);
            notify_start(&client, &addr, game_state)
        })
    ).await;

    let start_errs: Vec<String> = start_responses.iter().filter_map(|result| result.err()).collect();
    if !start_errs.is_empty() {
        for msg in start_errs.iter() {
            error!("{}", msg);
        }
        error!("Some snake(s) failed to respond to the start call. Shutting down");
        return;
    }

    let live_snakes: Mutex<Vec<LiveSnake>> = Mutex::new(start_responses.iter().map)

    while turn.snakes.len() > 1 {
        info!("Requesting moves for turn {}", turn_index);
        let snake_moves = turn.snakes.iter().enumerate().map(|(i, snake)| {
            let game_state = build_api_game_state(&turn, i, turn_index, &game_id);
            // let req = Request::post(format!("{}/move", addr))
            //     .body(Body::from(serde_json::to_string(&game_state).unwrap()))
            //     .unwrap();
            // if let Ok(res) = client.request(req).await {
            //
            // } else {
            //
            // }
        }).collect::<Vec<_>>();

        // let dead_snake_indices = turn.advance(true, &snake_moves);

        //notify dead snakes about /end
        turn_index += 1;
    }

    //notify winner (may be none if both died in final turn)
}

fn build_api_game_state(turn: &Turn, snake_index: usize, turn_index: u32, game_id: &str) -> ApiGameState {
    ApiGameState {
        game: ApiGame {id: String::from(game_id)},
        turn: turn_index,
        board: ApiBoard {
            height: turn.height() as u32,
            width: turn.width() as u32,
            food: turn.food.iter().map(ApiCoords::from).collect(),
            snakes: turn.snakes.iter()
                .enumerate()
                .filter_map(|(i, snake)| {
                    if i != snake_index {
                        Some(build_api_snake(snake, "id", "name"))
                    } else {
                        None
                    }
                })
                .collect()
        },
        you: build_api_snake(turn.snakes.get(snake_index).unwrap(), "id", "name"),
    }
}

fn build_api_snake(snake: &Snake, id: &str, name: &str) -> ApiSnake {
    ApiSnake {
        id: String::from(id),
        name: String::from(name),
        health: snake.health as u32,
        body: snake.body.nodes.iter().map(ApiCoords::from).collect()
    }
}
