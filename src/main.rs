mod api;
mod game;
mod brain;

use std::collections::HashMap;
use std::sync::Mutex;
use actix_web::{web, get, post, App, HttpServer, Responder};
use crate::api::{ApiSnakeConfig, ApiGameState, ApiMove, ApiGameId};
use crate::game::Game;
use crate::brain::get_decision;

struct AppState {
    snake_config: ApiSnakeConfig,
    games: Mutex<HashMap<ApiGameId, Game>>,
}

#[get("/")]
fn handle_index(app_state: web::Data<AppState>) -> impl Responder {
    web::Json(app_state.snake_config.clone())
}

//we have 5s to respond
#[post("/start")]
fn handle_start(app_state: web::Data<AppState>, game_state: web::Json<ApiGameState>) -> impl Responder {
    let games = &mut *app_state.games.lock().unwrap();
    games.insert(game_state.game.id.clone(), Game::init(&game_state));
    //todo: start a thread working on predictions
    web::Json(app_state.snake_config.clone())
}

//we have 500ms to respond
#[post("/move")]
fn handle_move(app_state: web::Data<AppState>, game_state: web::Json<ApiGameState>) -> impl Responder {
    let games = &mut *app_state.games.lock().unwrap();
    let direction = match games.get_mut(&game_state.game.id) {
        Some(mat) => {
            mat.update(&game_state);
            get_decision(mat)
        },
        None => {
            //maybe we missed the "/start" call?
            let mat = Game::init(&game_state);
            let decision = get_decision(&mat);
            games.insert(game_state.game.id.clone(), mat);
            decision
        }
    };
    web::Json(ApiMove {move_dir: direction})
}

#[post("/end")]
fn handle_end(app_state: web::Data<AppState>, game_state: web::Json<ApiGameState>) -> impl Responder {
    let games = &mut *app_state.games.lock().unwrap();
    games.remove(&game_state.game.id);
    "cya"
}

#[post("/ping")]
fn handle_ping() -> impl Responder {
    "pong"
}

fn main() {
    let app_state = web::Data::new(AppState {
        games: Mutex::new(HashMap::new()),
        snake_config: ApiSnakeConfig {
            color: String::from("#800080"),
            head_type: String::from("fang"),
            tail_type: String::from("round-bum"),
        },
    });

    HttpServer::new(move || {
        App::new()
            .register_data(app_state.clone())
            .service(handle_index)
            .service(handle_start)
            .service(handle_move)
            .service(handle_end)
            .service(handle_ping)
    })
    .bind("127.0.0.1:8080")
    .expect("Failed to bind")
    .run()
    .unwrap();
}
