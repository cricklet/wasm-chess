use itertools::Itertools;
use std::{iter, sync::Mutex};

use super::{
    game::Game,
    helpers::{err_result, prefix, ErrorResult},
    perft::run_perft_counting_first_move,
};

pub struct UciAsync {
    game: Mutex<Game>,
}

pub struct Uci {
    pub game: Game,
}

impl Uci {
    pub fn handle_line(&mut self, line: &str) -> Box<dyn Iterator<Item = ErrorResult<String>>> {
        if line.starts_with("position") {
            let game = Game::from_position_uci(line);
            if let Err(e) = &game {
                return Box::new(iter::once(Err(e.clone())));
            }
            self.game = game.unwrap();
            println!("Game: {}", self.game);
            Box::new(iter::empty())
        } else if line.starts_with("go perft") {
            let depth = line["go perft".len()..].trim();
            let depth = match depth.parse::<usize>() {
                Ok(depth) => depth,
                Err(_) => {
                    return Box::new(iter::once(err_result(&format!(
                        "invalid depth for '{}'",
                        line
                    ))));
                }
            };
            let perft_result = run_perft_counting_first_move(&self.game, depth);
            if let Err(e) = &perft_result {
                return Box::new(iter::once(Err(e.clone())));
            }
            let (perft_overall, perft_per_move) = perft_result.unwrap();
            let perft_per_move = perft_per_move
                .into_iter()
                .map(|(mv, count)| format!("{}: {}", mv, count));
            let perft_output = format!(
                "{}\nNodes searched: {}",
                perft_per_move.into_iter().join("\n"),
                perft_overall,
            );
            Box::new(iter::once(perft_output).map(Ok))
        } else if line == "d" {
            let debug_str = format!("{}\nFen: {}", self.game, self.game.to_fen());
            let debug_iter = iter::once(debug_str);
            Box::new(debug_iter.map(Ok))
        } else if line == "stop" {
            Box::new(iter::empty())
        } else {
            Box::new(iter::once(err_result(&format!(
                "Unknown command: '{}'",
                line
            ))))
        }
    }
}
