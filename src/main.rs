mod api;
mod game;
use actix_web::{web, get, post, http, App, HttpServer, Responder};
use api::{SnakeConfig, GameState, Move, Direction};

struct AppConfig {
    snake_config: SnakeConfig,
}

#[get("/")]
fn handle_index(config: web::Data<AppConfig>) -> impl Responder {
    web::Json(config.snake_config.clone())
}

#[post("/start")]
fn handle_start(config: web::Data<AppConfig>, _game_state: web::Json<GameState>) -> impl Responder {
    //todo: setup resources to handle this game
    web::Json(config.snake_config.clone())
}

#[post("/move")]
fn handle_move(_game_state: web::Json<GameState>) -> impl Responder {
    //todo: calculate move for game
    web::Json(Move {
        move_dir: Direction::Right
    })
}

#[post("/end")]
fn handle_end(_game_state: web::Json<GameState>) -> impl Responder {
    //todo: capture game result
    web::HttpResponse::build(http::StatusCode::OK)
}

#[post("/ping")]
fn handle_ping(_game_state: web::Json<GameState>) -> impl Responder {
    web::HttpResponse::build(http::StatusCode::OK)
}

fn main() {
    HttpServer::new(|| {
        App::new()
            .service(handle_index)
            .service(handle_start)
            .service(handle_move)
            .service(handle_end)
            .service(handle_ping)
            .data(AppConfig {
                snake_config: SnakeConfig {
                    color: String::from("#800080"),
                    head_type: String::from("fang"),
                    tail_type: String::from("round-bum"),
                }
            })
    })
    .bind("127.0.0.1:8080")
    .expect("Failed to bind port")
    .run()
    .unwrap();
}
