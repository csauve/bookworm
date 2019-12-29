use crate::api::{Snake, Coords, GameState};

//todo: can state be broken up in a way that allows memoization, avoiding cycles?
//todo: store probabilities and scores in the structure; update when invalidated?

pub enum NextTurn {
    Unknown,
    Known(Box<Turn>),
    Speculative(Vec<Turn>),
}

pub struct Turn {
    pub turn: u32,
    pub you: Snake,
    pub enemies: Vec<Snake>,
    pub food: Vec<Coords>,
    pub next: NextTurn,
}

pub struct Match {
    pub width: u32,
    pub height: u32,
    pub start: Turn,
}

impl Turn {
    pub fn init(game_state: &GameState) -> Turn {
        Turn {
            turn: game_state.turn,
            you: game_state.you.clone(),
            enemies: game_state.board.snakes.clone(),
            food: game_state.board.food.clone(),
            next: NextTurn::Unknown,
        }
    }
}

impl Match {
    pub fn init(game_state: &GameState) -> Match {
        Match {
            width: game_state.board.width,
            height: game_state.board.height,
            start: Turn::init(game_state),
        }
    }

    fn get_turn_mut(&mut self, num: u32) -> Option<&mut Turn> {
        let mut current = &self.start;
        // loop {
        //     ...
        //     match current.next {
        //     }
        // }
        Option::None
    }

    pub fn update(&mut self, _game_state: &GameState) {
        //todo
    }
}
