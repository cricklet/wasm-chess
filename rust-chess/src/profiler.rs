#![allow(dead_code)]
#![allow(unused_imports)]
#![cfg(feature = "profiling")]

use std::{
    cell::RefCell, fs::File, future::Future, io::Write, pin::Pin, sync::Arc, time::Duration,
};

use num_format::{Locale, ToFormattedString};
use pprof::protos::Message;

pub mod shared;
pub use shared::*;

use crate::{
    alphabeta::{AlphaBetaOptions, AlphaBetaStack, LoopResult},
    game::Game,
    iterative_deepening::{IterativeSearch, IterativeSearchOptions},
    perft::run_perft_iteratively_to_depth,
    perft::run_perft_recursively,
    transposition_table::TranspositionTable,
    traversal::null_move_sort,
};

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

pub fn profile_transposition_table() {
    // let fen = "startpos";
    // let max_depth = 7;

    // mid-game fen
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";
    let max_depth = 6;

    // // late-game fen
    // let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";
    // let max_depth = 8;

    // make sure any lazy-statics are generated
    IterativeSearch::new(
        Game::from_fen(fen).unwrap(),
        IterativeSearchOptions::default(),
    )
    .unwrap()
    .iterate(&mut |_| {})
    .unwrap();

    let options = IterativeSearchOptions {
        transposition_table: Some(RefCell::new(TranspositionTable::new())),
        ..IterativeSearchOptions::default()
    };

    let mut search = IterativeSearch::new(Game::from_fen(fen).unwrap(), options.clone()).unwrap();

    println!("{}", options);

    let start_time = std::time::Instant::now();

    let mut last_log_time = std::time::Instant::now();
    let mut log_callback = |line: &str| {
        println!(
            "{:>5} ms {}",
            last_log_time
                .elapsed()
                .as_millis()
                .to_formatted_string(&Locale::en),
            line.to_string()
        );
        last_log_time = std::time::Instant::now();
    };

    let p = Profiler::new("transposition_table".to_string());

    loop {
        search.iterate(&mut log_callback).unwrap();
        if search.max_depth() >= max_depth {
            break;
        }
        if start_time.elapsed() > std::time::Duration::from_secs(5) {
            break;
        }
    }

    p.flush();

    let total_time = start_time.elapsed();
    println!(
        "{:>5} ms total",
        total_time.as_millis().to_formatted_string(&Locale::en)
    );
}

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let expected_count = [
        1, 20, 400, 8902, 197281, 4865609, 119060324,
        // 3195901860,
    ];

    // setup magics
    run_perft_recursively(Game::from_fen(fen).unwrap(), 2).unwrap();

    println!("\ntransposition-table");
    {
        profile_transposition_table();
    }

    println!("\nalpha-beta");
    {
        let p = Profiler::new("alpha_beta".to_string());

        let start_time = std::time::Instant::now();
        let mut search = AlphaBetaStack::with(
            Game::from_fen("startpos").unwrap(),
            5,
            AlphaBetaOptions::default(),
        )
        .unwrap();
        loop {
            match search.iterate(null_move_sort).unwrap() {
                LoopResult::Continue => {}
                LoopResult::Done => break,
            }
        }

        println!(
            "search found: {:#?} in {} ms",
            search.bestmove(),
            (std::time::Instant::now() - start_time).as_millis()
        );

        p.flush();
    }

    println!("\nrecursive");

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
