use serde::{Serialize, Deserialize};

pub type ApiGameId = String;
pub type ApiSnakeId = String;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiSnakeConfig {
    pub color: String,
    pub head_type: String,
    pub tail_type: String,
}

#[derive(Deserialize)]
pub struct ApiGame {
    pub id: ApiGameId,
}

#[derive(Deserialize, Copy, Clone, PartialEq)]
pub struct ApiCoords {
    pub x: u32,
    pub y: u32,
}

#[derive(Deserialize)]
pub struct ApiBoard {
    pub height: u32,
    pub width: u32,
    pub food: Vec<ApiCoords>,
    pub snakes: Vec<ApiSnake>,
}

#[derive(Deserialize, Clone)]
pub struct ApiSnake {
    pub id: ApiSnakeId,
    pub name: String,
    pub health: u32,
    pub body: Vec<ApiCoords>,
}

#[derive(Deserialize)]
pub struct ApiGameState {
    pub game: ApiGame,
    pub turn: u32,
    pub board: ApiBoard,
    pub you: ApiSnake,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize)]
pub struct ApiMove {
    #[serde(rename = "move")]
    pub move_dir: ApiDirection,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    impl ApiSnake {
        pub fn spawn(name: &str, x: u32, y: u32) -> ApiSnake {
            ApiSnake {
                id: format!("snake_{}", name),
                name: String::from(name),
                health: 100,
                body: vec![ApiCoords {x, y}]
            }
        }
    }
}
