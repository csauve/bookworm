use std::collections::HashMap;
use std::sync::Mutex;
use actix_web::{web, get, post, App, HttpServer, Responder};
use crate::api::{ApiSnakeConfig, ApiGameState, ApiMove, ApiGameId};
use crate::game::Game;

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
        Some(game) => {
            game.update(&game_state);
            game.get_decision()
        },
        None => {
            //maybe we missed the "/start" call?
            let game = Game::init(&game_state);
            let decision = game.get_decision();
            games.insert(game_state.game.id.clone(), game);
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

pub fn start_server(ip: &str, port: &str) {
    let bind_to = format!("{}:{}", ip, port);

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
    .bind(bind_to.clone())
    .unwrap_or_else(|_| panic!("Failed to bind to {}", bind_to))
    .run()
    .unwrap();
}
