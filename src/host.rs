use std::time::Duration;
use std::io;
use futures::{future, FutureExt};
use log::*;
use tokio::time::timeout;
use uuid::Uuid;
use hyper::{Client, Request, Body, body, client::connect::HttpConnector};
use crate::game::{Board, Snake, UnitAbs};
use crate::api::*;
use crate::util::draw_board;

const START_TIMEOUT_MS: u64 = 5000;

#[derive(Clone)]
struct LiveSnake {
    pub id: ApiSnakeId,
    pub addr: String,
    pub config: ApiSnakeConfig,
}

async fn notify_start(client: &Client<HttpConnector>, addr: &str, game_state: ApiGameState) -> Result<ApiSnakeConfig, String> {
    let req = Request::post(format!("{}/start", addr))
        .body(Body::from(serde_json::to_string(&game_state).unwrap()))
        .unwrap();
    let k = timeout(Duration::from_millis(START_TIMEOUT_MS), client.request(req));
    match k.await {
        Err(_) => {
            Err(format!("Snake @ {} timed out after {} ms", addr, START_TIMEOUT_MS))
        },
        Ok(Err(e)) => {
            Err(format!("Snake @ {} failed to reply: {}", addr, e))
        },
        Ok(Ok(res)) => {
            let res_body = body::to_bytes(res.into_body());
            match serde_json::from_slice::<ApiSnakeConfig>(&res_body.await.unwrap()) {
                Ok(start_response) => Ok(start_response),
                Err(e) => Err(format!("Snake @ {} responded with invalid JSON: {}", addr, e))
            }
        }
    }
}

async fn get_move(client: &Client<HttpConnector>, addr: String, game_state: ApiGameState, timeout_ms: u64) -> Result<ApiMove, String> {
    let req = Request::post(format!("{}/move", addr))
        .body(Body::from(serde_json::to_string(&game_state).unwrap()))
        .unwrap();
    match timeout(Duration::from_millis(timeout_ms), client.request(req)).await {
        Err(_) => {
            Err(format!("Snake @ {} timed out after {} ms", addr, timeout_ms))
        },
        Ok(Err(e)) => {
            Err(format!("Snake @ {} failed to reply: {}", addr, e))
        },
        Ok(Ok(res)) => {
            let res_body = body::to_bytes(res.into_body());
            match serde_json::from_slice::<ApiMove>(&res_body.await.unwrap()) {
                Ok(move_response) => Ok(move_response),
                Err(e) => Err(format!("Snake @ {} responded with invalid JSON: {}", addr, e))
            }
        }
    }
}

pub async fn run_game(timeout_ms: u64, snakes_addrs: &[String], width: UnitAbs, height: UnitAbs, prompt: bool) {
    info!("Initializing {}x{} board", width, height);
    let mut board = Board::init(width, height, snakes_addrs.len()).unwrap();
    let mut turn: u32 = 0;
    let game_id: ApiGameId = Uuid::new_v4().to_string();
    let client = Client::default();

    info!("Notifying snakes of game start; id: {}", &game_id);
    let live_snakes = future::try_join_all(
        //build an iterator of futures representing results of /start API call
        snakes_addrs.iter().enumerate().map(|(snake_index, addr)| {
            let game_state = build_api_game_state(&board, snake_index, turn, &game_id);
            let addr_copy = addr.clone();
            //within the future, within the result, wrap their response in a LiveSnake
            notify_start(&client, &addr, game_state).map(move |call_result| {
                call_result.map(|conf| {
                    LiveSnake {
                        id: Uuid::new_v4().to_string(),
                        addr: addr_copy,
                        config: conf
                    }
                })
            })
        })
    ).await;

    if let Err(e) = live_snakes {
        error!("Some snake(s) failed to respond to the start call: {}", &e);
        return;
    }
    let mut live_snakes: Vec<LiveSnake> = live_snakes.unwrap();

    while board.snakes.len() > 1 {
        info!("Turn {}: {} snakes\n{}", turn, board.snakes.len(), draw_board(&board));
        if prompt {
            wait_for_prompt();
        }

        info!("Requesting moves for turn {}. Snakes have {} ms to respond", turn, timeout_ms);
        let snake_moves = future::join_all(
            board.snakes.iter().enumerate().map(|(snake_index, snake)| {
                let default_move = snake.get_default_move();
                let game_state = build_api_game_state(&board, snake_index, turn, &game_id);
                let addr_copy = live_snakes.get(snake_index).unwrap().addr.clone();
                get_move(&client, addr_copy, game_state, timeout_ms).map(move |call_result| {
                    call_result.map(|api_move| api_move.decision).unwrap_or_else(|err| {
                        warn!("Using default move for snakes: {}", &err);
                        default_move
                    })
                })
            })
        ).await;

        let dead_snake_indices = board.advance(true, &snake_moves);

        //todo: notify dead snakes about /end
        if !dead_snake_indices.is_empty() {
            info!("Snakes died: {:?}", &dead_snake_indices);
            live_snakes = live_snakes.iter().enumerate()
                .filter_map(|(i, ls)| {
                    if dead_snake_indices.contains_key(&i) {
                        None
                    } else {
                        Some(ls.clone())
                    }
                })
                .collect();
        }

        turn += 1;
    }

    info!("Game has ended!");
    //notify winner (may be none if both died in final turn)
}



fn wait_for_prompt() {
    info!("Press [ENTER] to continue");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn build_api_game_state(board: &Board, snake_index: usize, turn: u32, game_id: &str) -> ApiGameState {
    ApiGameState {
        game: ApiGame {id: String::from(game_id)},
        turn,
        board: ApiBoard {
            height: board.height() as u32,
            width: board.width() as u32,
            food: board.food.iter().map(ApiCoords::from).collect(),
            snakes: board.snakes.iter()
                .enumerate()
                .filter_map(|(i, snake)| {
                    if i != snake_index {
                        Some(build_api_snake(snake, &format!("id_{}", snake_index), &format!("name_{}", snake_index)))
                    } else {
                        None
                    }
                })
                .collect()
        },
        you: build_api_snake(board.snakes.get(snake_index).unwrap(), &format!("id_{}", snake_index), &format!("name_{}", snake_index)),
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
