use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SnakeConfig {
    pub color: String,
    pub head_type: String,
    pub tail_type: String,
}

#[derive(Deserialize)]
pub struct Game {
    pub id: String,
}

#[derive(Deserialize)]
pub struct Coords {
    pub x: u32,
    pub y: u32,
}

#[derive(Deserialize)]
pub struct Board {
    pub height: u32,
    pub width: u32,
    pub food: Vec<Coords>,
    pub snakes: Vec<Snake>,
}

#[derive(Deserialize)]
pub struct Snake {
    pub id: String,
    pub name: String,
    pub health: u32,
    pub body: Vec<Coords>,
}

#[derive(Deserialize)]
pub struct GameState {
    pub game: Game,
    pub turn: u32,
    pub board: Board,
    pub you: Snake,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize)]
pub struct Move {
    #[serde(rename = "move")]
    pub move_dir: Direction,
}
