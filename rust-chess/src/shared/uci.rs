use itertools::Itertools;
use std::{iter, sync::Mutex};

use crate::{
    alphabeta::{LoopResult, AlphaBetaStack},
    bitboard::warm_magic_cache,
    iterative_deepening::{IterativeSearch, IterativeSearchOptions},
};

use super::{
    game::Game,
    helpers::{err_result, ErrorResult},
    perft::run_perft_counting_first_move,
};

pub struct UciAsync {
    game: Mutex<Game>,
}

pub struct Uci {
    pub game: Game,
    pub search: Option<IterativeSearch>,
}

impl Uci {
    pub fn new() -> Self {
        Self {
            game: Game::from_position_uci(&"position startpos").unwrap(),
            search: None,
        }
    }
    pub fn handle_line(&mut self, line: &str) -> ErrorResult<String> {
        if line.starts_with("position") {
            let game = Game::from_position_uci(line);
            if let Err(e) = &game {
                return Err(e.clone());
            }
            self.game = game.unwrap();
            Ok(format!("{:?}", self.game))
        } else if line.starts_with("go perft") {
            let depth = line["go perft".len()..].trim();
            let depth = match depth.parse::<usize>() {
                Ok(depth) => depth,
                Err(_) => {
                    return err_result(&format!("invalid depth for '{}'", line));
                }
            };
            let perft_result = run_perft_counting_first_move(&self.game, depth);
            if let Err(e) = &perft_result {
                return Err(e.clone());
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
            Ok(perft_output)
        } else if line == "d" {
            let debug_str = format!("{}\nFen: {}", self.game, self.game.to_fen());
            Ok(debug_str)
        } else if line == "go" {
            let search = IterativeSearch::new(self.game, IterativeSearchOptions::default())?;
            self.search = Some(search);
            Ok("".to_string())
        } else if line == "stop" {
            self.finish_search()
        } else {
            err_result(&format!("Unknown command: '{}'", line))
        }
    }

    fn finish_search(&mut self) -> ErrorResult<String> {
        if let Some(search) = &mut self.search {
            let best_move = search.bestmove();
            self.search = None;

            match best_move {
                Some((best_move, response_moves)) => Ok(format!(
                    "bestmove {} ponder {}",
                    best_move.to_uci(),
                    response_moves.iter().map(|v| v.to_uci()).join(" ")
                )),
                None => Ok("bestmove (none)".to_string()),
            }
        } else {
            Ok("".to_string())
        }
    }

    pub fn think(&mut self) -> ErrorResult<String> {
        let mut output: Vec<String> = vec![];
        for _ in 0..100_000 {
            if let Some(search) = &mut self.search {
                search.iterate(&mut |line| output.push(line.to_string()))?;
            }
        }

        Ok(output.join("\n"))
    }
}
