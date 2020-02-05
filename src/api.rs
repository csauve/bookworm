use std::cmp::max;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub type ApiGameId = String;
pub type ApiSnakeId = String;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiSnakeConfig {
    pub color: String,
    pub head_type: String,
    pub tail_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct ApiGame {
    pub id: ApiGameId,
}

#[derive(Deserialize, Serialize, Copy, Clone, PartialEq, Debug)]
pub struct ApiCoords {
    pub x: u32,
    pub y: u32,
}

#[derive(Deserialize, Serialize)]
pub struct ApiBoard {
    pub height: u32,
    pub width: u32,
    pub food: Vec<ApiCoords>,
    pub snakes: Vec<ApiSnake>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ApiSnake {
    pub id: ApiSnakeId,
    pub name: String,
    pub health: u32,
    pub body: Vec<ApiCoords>,
}

#[derive(Deserialize, Serialize)]
pub struct ApiGameState {
    pub game: ApiGame,
    pub turn: u32,
    pub board: ApiBoard,
    pub you: ApiSnake,
}

#[derive(Deserialize, Serialize, Copy, Clone, PartialEq, Debug, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ApiDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ApiDirection {
    #[inline]
    pub fn as_index(self) -> usize {
        match self {
            Self::Down => 0,
            Self::Left => 1,
            Self::Up => 2,
            Self::Right => 3,
        }
    }
}

pub const ALL_DIRS: [ApiDirection; 4] = [ApiDirection::Down, ApiDirection::Left, ApiDirection::Up, ApiDirection::Right];

#[derive(Deserialize, Serialize)]
pub struct ApiMove {
    #[serde(rename = "move")]
    pub decision: ApiDirection,
}

impl ApiGameState {
    pub fn parse_basic(s: &str) -> ApiGameState {
        let mut height = 0;
        let mut width = 0;
        let mut food = Vec::new();
        let mut snakes_coords: HashMap<String, Vec<ApiCoords>> = HashMap::new();
        let mut snake_health: HashMap<String, u32> = HashMap::new();
        let mut you_coords = Vec::new();

        for row in s.lines().map(str::trim) {
            if row.starts_with('|') {
                let cols = row.trim_start_matches('|').split_terminator('|').collect::<Vec<_>>();
                width = std::cmp::max(width, cols.len());
                for (x, &col) in cols.iter().enumerate() {
                    let coord = ApiCoords {x: x as u32, y: height as u32};
                    match col.trim() {
                        "" => {},
                        "()" => {
                            food.push(coord);
                        },
                        content => {
                            if content.is_empty() {
                                continue;
                            }
                            let snake_name: String = content.chars().take_while(|&c| c.is_alphabetic()).collect();
                            let index: usize = content.chars().skip_while(|&c| c.is_alphabetic()).collect::<String>().parse().unwrap();
                            if snake_name == "Y" {
                                you_coords.resize(max(you_coords.len(), index + 1), coord);
                                you_coords[index] = coord;
                            } else if let Some(body) = snakes_coords.get_mut(&snake_name) {
                                body.resize(max(body.len(), index + 1), coord);
                                body[index] = coord;
                            } else {
                                let mut body = Vec::new();
                                body.resize(max(body.len(), index + 1), coord);
                                body[index] = coord;
                                snakes_coords.insert(snake_name, body);
                            }
                        }
                    }
                }
                height += 1;
            } else if row.starts_with('+') {
                let snake_name: String = row.chars().skip(1).take_while(|&c| c.is_alphabetic()).collect();
                let health: u32 = row.chars().skip(1).skip_while(|&c| c.is_alphabetic()).collect::<String>().parse().unwrap();
                snake_health.insert(snake_name, health);
            }
        }

        ApiGameState {
            game: ApiGame {id: ApiGameId::from("123")},
            turn: 0,
            board: ApiBoard {
                height: height as u32,
                width: width as u32,
                food,
                snakes: snakes_coords.iter().map(|(name, body)| ApiSnake {
                    id: format!("id_{}", name),
                    name: name.clone(),
                    health: snake_health.get(name).copied().unwrap_or(100),
                    body: body.clone()
                }).collect(),
            },
            you: ApiSnake {
                id: String::from("id_Y"),
                name: String::from("Y"),
                health: 100,
                body: you_coords,
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let game = ApiGameState::parse_basic("
        |  |  |  |  |  |
        |Y8|Y7|Y6|Y5|  |
        |  |Y0|  |Y4|  |
        |  |Y1|Y2|Y3|  |
        |  |  |  |  |  |
        ");

        assert_eq!(game.you.body.len(), 9);
    }
}
