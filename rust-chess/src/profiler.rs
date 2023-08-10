#![allow(dead_code)]
#![allow(unused_imports)]

use std::{fs::File, io::Write};

use pprof::protos::Message;

pub mod alphabeta;
pub mod bitboard;
pub mod danger;
pub mod evaluation;
pub mod game;
pub mod helpers;
pub mod iterative_traversal;
pub mod moves;
pub mod perft;
pub mod types;
pub mod uci;

use {game::Game, perft::run_perft_iteratively_to_depth, perft::run_perft_recursively};

pub struct Profiler<'a> {
    name: String,
    guard: pprof::ProfilerGuard<'a>,
}

impl<'a> Profiler<'a> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            guard: pprof::ProfilerGuardBuilder::default()
                .frequency(1000)
                .blocklist(&["libc", "libgcc", "pthread", "vdso", "backtrace"])
                .build()
                .unwrap(),
        }
    }

    pub fn flush(&self) {
        match self.guard.report().build() {
            Ok(report) => {
                let mut file = File::create(&format!("profiles/{}.pb", self.name)).unwrap();
                let profile = report.pprof().unwrap();

                let mut content = Vec::new();
                profile.write_to_vec(&mut content).unwrap();

                file.write_all(&content).unwrap();
            }
            Err(_) => {}
        };
    }
}

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let expected_count = [
        1, 20, 400, 8902, 197281, 4865609, 119060324,
        // 3195901860,
    ];

    println!("recursive");

    run_perft_recursively(Game::from_fen(fen).unwrap(), 2).unwrap();
    {
        // let p = Profiler::new("recursive_perft".to_string());
        for (i, expected_count) in expected_count.into_iter().enumerate() {
            let start_time = std::time::Instant::now();

            let max_depth = i;

            let count = run_perft_recursively(Game::from_fen(fen).unwrap(), max_depth).unwrap();
            let end_time = std::time::Instant::now();

            if count != expected_count {
                panic!(
                    "wrong count for max_depth: {}, expected_count: {}, in {} ms",
                    max_depth,
                    expected_count,
                    (end_time - start_time).as_millis()
                );
            }

            println!(
                "calculated recursive perft for max_depth: {}, expected_count: {}, in {} ms",
                max_depth,
                expected_count,
                (end_time - start_time).as_millis()
            );
        }
        // p.flush();
    }

    println!("\niterative");

    run_perft_iteratively_to_depth(Game::from_fen(fen).unwrap(), 2).unwrap();
    {
        // let p = Profiler::new("iterative_perft".to_string());
        for (i, expected_count) in expected_count.into_iter().enumerate() {
            let start_time = std::time::Instant::now();

            let max_depth = i + 1;

            let count =
                run_perft_iteratively_to_depth(Game::from_fen(fen).unwrap(), max_depth).unwrap();
            let end_time = std::time::Instant::now();

            if count != expected_count {
                panic!(
                    "wrong count for max_depth: {}, expected_count: {}, in {} ms",
                    max_depth,
                    expected_count,
                    (end_time - start_time).as_millis()
                );
            }

            println!(
                "calculated iterative perft for max_depth: {}, expected_count: {}, in {} ms",
                max_depth,
                expected_count,
                (end_time - start_time).as_millis()
            );
        }
        // p.flush();
    }
}
