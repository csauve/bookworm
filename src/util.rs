use std::iter;
use ansi_term::{Colour, Style};
use crate::game::{Board, Coord};

const SNAKE_COLOURS: [Colour; 6] = [
    Colour::Red,
    Colour::Purple,
    Colour::Blue,
    Colour::Green,
    Colour::Yellow,
    Colour::Cyan,
];


//todo: try returning an iterator instead to avoid allocating the vec if caller doesnt need it
pub fn cartesian_product<T: Copy>(lists: &[Vec<T>]) -> Vec<Vec<T>> {
    lists.iter().fold(vec![vec![]], |product, list| {
        list.iter().flat_map(|item| {
            product.iter().map(|prev_tuple| {
                let mut new_tuple = prev_tuple.clone();
                new_tuple.push(*item);
                new_tuple
            }).collect::<Vec<Vec<T>>>()
        }).collect::<Vec<Vec<T>>>()
    })
}

pub fn draw_board(board: &Board) -> String {
    let w = board.width();
    let h = board.height();

    let mut grid = iter::repeat_with(|| {
        iter::repeat_with(|| {
            String::from(" ")
        }).take(w).collect::<Vec<_>>()
    }).take(h).collect::<Vec<_>>();

    for &Coord {x, y} in board.food.iter() {
        grid[y as usize][x as usize] = String::from("*");
    }

    for (snake_i, snake) in board.snakes.iter().enumerate() {
        for (body_i, &Coord {x, y}) in snake.body.nodes.iter().enumerate() {
            let mut style = Style::from(SNAKE_COLOURS[snake_i % SNAKE_COLOURS.len()]);
            if body_i == 0 {
                style = style.underline();
            }
            grid[y as usize][x as usize] = style.paint(snake_i.to_string()).to_string();
        }
    }

    let mut buf = String::new();
    buf.push_str(&(0..=(w * 4)).map(|i| {
        if i == 0 {
            Colour::Black.paint("╔").to_string()
        } else if i == w * 4 {
            Colour::Black.paint("╗\n").to_string()
        } else if i % 4 == 0 {
            Colour::Black.paint("╤").to_string()
        } else {
            Colour::Black.paint("═").to_string()
        }
    }).collect::<String>());

    for (i, row) in grid.iter().enumerate() {
        if i != 0 {
            buf.push_str(&(0..=(w * 4)).map(|i| {
                if i == 0 {
                    Colour::Black.paint("╟").to_string()
                } else if i == w * 4 {
                    Colour::Black.paint("╢\n").to_string()
                } else if i % 4 == 0 {
                    Colour::Black.paint("┼").to_string()
                } else {
                    Colour::Black.paint("─").to_string()
                }
            }).collect::<String>());
        }
        buf.push_str(&Colour::Black.paint("║ ").to_string());
        buf.push_str(&row.join(&Colour::Black.paint(" │ ").to_string()));
        buf.push_str(&Colour::Black.paint(" ║\n").to_string());
    }

    buf.push_str(&(0..=(w * 4)).map(|i| {
        if i == 0 {
            Colour::Black.paint("╚").to_string()
        } else if i == w * 4 {
            Colour::Black.paint("╝").to_string()
        } else if i % 4 == 0 {
            Colour::Black.paint("╧").to_string()
        } else {
            Colour::Black.paint("═").to_string()
        }
    }).collect::<String>());

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test() {
        let result = cartesian_product(&[
            vec!["1", "2"],
            vec!["a", "b"],
            vec!["x", "y", "z"],
        ]);

        assert_eq!(result, &[
            ["1", "a", "x"],
            ["2", "a", "x"],
            ["1", "b", "x"],
            ["2", "b", "x"],
            ["1", "a", "y"],
            ["2", "a", "y"],
            ["1", "b", "y"],
            ["2", "b", "y"],
            ["1", "a", "z"],
            ["2", "a", "z"],
            ["1", "b", "z"],
            ["2", "b", "z"],
        ]);
    }
}
