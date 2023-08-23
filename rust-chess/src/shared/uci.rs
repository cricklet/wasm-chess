use itertools::Itertools;
use std::{cell::RefCell, iter, rc::Rc, sync::Mutex};

use crate::{
    alphabeta::{AlphaBetaStack, LoopResult},
    bitboard::warm_magic_cache,
    fen::FenDefinition,
    helpers::Joinable,
    iterative_deepening::{IterativeSearch, IterativeSearchOptions},
    transposition_table::TranspositionTable,
    zobrist::{ZobristHistory, IsDraw},
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
    pub tt: Rc<RefCell<TranspositionTable>>,
    pub history: ZobristHistory,
}

impl Uci {
    pub fn new() -> Self {
        Self {
            game: Game::from_position_uci(&"position startpos").unwrap(),
            search: None,
            tt: Rc::new(RefCell::new(TranspositionTable::new())),
            history: ZobristHistory::new(),
        }
    }
    pub fn handle_line(&mut self, line: &str) -> ErrorResult<String> {
        if line.starts_with("position") {
            let (position_str, moves) = FenDefinition::split_uci(line)?;
            let game = Game::from_position_and_moves(&position_str, &moves);
            if let Err(e) = &game {
                return Err(e.clone());
            }
            self.game = game.unwrap();

            if self.history.update(position_str, &moves) == IsDraw::Yes {
                Ok(format!("draw detected\n{:?}", self.game))
            } else {
                Ok(format!("{:?}", self.game))
            }
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
            let search = IterativeSearch::new(
                self.game,
                IterativeSearchOptions {
                    transposition_table: Some(self.tt.clone()),
                    starting_history: self.history.clone(),
                    ..IterativeSearchOptions::default()
                },
            )?;
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
                    best_move,
                    response_moves.iter().map(|v| v.to_string()).join(" ")
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

#[test]
fn test_match_50ms() {
    let mut uci = Uci::new();
    let mut moves: Vec<String> = vec![];

    loop {
        let position_uci_line = format!("position startpos moves {}", moves.join(" "));
        println!("{}", position_uci_line);
        uci.handle_line(position_uci_line.as_str()).unwrap();

        let start = std::time::Instant::now();
        uci.handle_line("go").unwrap();

        let mut log = vec![];

        loop {
            log.push(uci.think().unwrap());
            if start.elapsed().as_millis() > 50 {
                break;
            }
        }

        println!(
            "{}",
            log.iter()
                .filter(|&l| !l.is_empty())
                .collect::<Vec<_>>()
                .join_vec(", ")
        );

        let result = uci.finish_search().unwrap();
        println!("{}", result);
        println!("{}", uci.handle_line("d").unwrap());

        let bestmove = result.split_whitespace().nth(1).unwrap();
        if bestmove.contains("none") {
            break;
        }

        moves.push(bestmove.to_string());
    }
}

#[test]
fn test_match_100ms() {
    let mut uci = Uci::new();
    let mut moves: Vec<String> = vec![];

    loop {
        let position_uci_line = format!("position startpos moves {}", moves.join(" "));
        println!("{}", position_uci_line);
        uci.handle_line(position_uci_line.as_str()).unwrap();

        let start = std::time::Instant::now();
        uci.handle_line("go").unwrap();

        let mut log = vec![];

        loop {
            log.push(uci.think().unwrap());
            if start.elapsed().as_millis() > 100 {
                break;
            }
        }

        println!(
            "{}",
            log.iter()
                .filter(|&l| !l.is_empty())
                .collect::<Vec<_>>()
                .join_vec(", ")
        );

        let result = uci.finish_search().unwrap();
        println!("{}", result);
        println!("{}", uci.handle_line("d").unwrap());

        let bestmove = result.split_whitespace().nth(1).unwrap();
        if bestmove.contains("none") {
            break;
        }

        moves.push(bestmove.to_string());
    }
}

#[test]
fn test_match_1000ms() {
    let mut uci = Uci::new();
    let mut moves: Vec<String> = vec![];

    loop {
        let position_uci_line = format!("position startpos moves {}", moves.join(" "));
        println!("{}", position_uci_line);
        uci.handle_line(position_uci_line.as_str()).unwrap();

        let start = std::time::Instant::now();
        uci.handle_line("go").unwrap();

        let mut log = vec![];

        loop {
            log.push(uci.think().unwrap());
            if start.elapsed().as_millis() > 1000 {
                break;
            }
        }

        println!(
            "{}",
            log.iter()
                .filter(|&l| !l.is_empty())
                .collect::<Vec<_>>()
                .join_vec(", ")
        );

        let result = uci.finish_search().unwrap();
        println!("{}", result);
        println!("{}", uci.handle_line("d").unwrap());

        let bestmove = result.split_whitespace().nth(1).unwrap();
        if bestmove.contains("none") {
            break;
        }

        moves.push(bestmove.to_string());
    }
}

#[test]
fn test_match_avoid_draw() {
    let mut uci = Uci::new();

    let mut moves: Vec<String> = vec![
        "d2d4", "d7d5", "b1c3", "b8c6", "g1f3", "g8f6", "c1g5", "f6e4", "e2e3", "e4g5", "f3g5",
        "e7e5", "f2f4", "f7f6", "g5f3", "e5e4", "f3d2", "c8e6", "d2e4", "d5e4", "d4d5", "e6d5",
        "c3d5", "f8d6", "g2g3", "d8d7", "f1g2", "f6f5", "d1d2", "e8c8", "e1c1", "c6e7", "d2a5",
        "e7c6", "a5d2", "a7a5", "h1e1", "c6b4", "d5b4", "a5b4", "c2c3", "d7e6", "d2d5", "e6d5",
        "d1d5", "g7g6", "c3b4", "d6b4", "d5d8", "h8d8", "e1d1", "d8d6", "d1d6", "c7d6", "b2b3",
        "d6d5", "a2a4", "h7h5", "c1d1", "b4c3", "g2f1", "c8d8", "h2h4", "b7b6", "f1b5", "d8e7",
        "b5c6", "d5d4", "e3d4", "c3d4", "d1e2", "e7f6", "b3b4", "f6e7", "e2f1", "e7e6", "b4b5",
        "e6e7", "c6d5", "e4e3", "d5b3", "e7f6",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    loop {
        let position_uci_line = format!("position startpos moves {}", moves.join(" "));
        println!("{}", position_uci_line);

        let result = uci.handle_line(position_uci_line.as_str()).unwrap();
        println!("{}", result);
        if result.contains("draw detected") {
            break;
        }

        let start = std::time::Instant::now();
        uci.handle_line("go").unwrap();

        let mut log = vec![];

        loop {
            log.push(uci.think().unwrap());
            if start.elapsed().as_millis() > 100 {
                break;
            }
        }

        println!(
            "{}",
            log.iter()
                .filter(|&l| !l.is_empty())
                .collect::<Vec<_>>()
                .join_vec(", ")
        );

        let result = uci.finish_search().unwrap();
        println!("{}", result);

        let bestmove = result.split_whitespace().nth(1).unwrap();
        if bestmove.contains("none") {
            break;
        }

        moves.push(bestmove.to_string());
    }
}
