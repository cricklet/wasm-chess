#![allow(dead_code)]
#![allow(unused_imports)]

use std::{fs::File, future::Future, io::Write, pin::Pin, sync::Arc, time::Duration};

use num_format::{Locale, ToFormattedString};
use pprof::protos::Message;

pub mod shared;
pub use shared::*;

use crate::async_perft::AsyncPerftRunner;

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

fn log_fn(s: &str) {
    println!("{}", s);
}

fn yield_fn() -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(tokio::time::sleep(Duration::from_millis(1)))
}

fn done_fn(count: usize) {
    println!("done: {}", count);
}

#[tokio::main]
async fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let expected_count = [
        1, 20, 400, 8902, 197281, 4865609, 119060324,
        // 3195901860,
    ];

    run_perft_recursively(Game::from_fen(fen).unwrap(), 2).unwrap();

    println!("\nasync");
    {
        let p = Profiler::new("async_perft_long".to_string());
        let perft = Arc::new(AsyncPerftRunner::from(yield_fn, log_fn, done_fn));

        let spawn_perft = perft.clone();
        tokio::spawn(async move {
            spawn_perft.start("startpos".to_string(), 7).await;
        });

        tokio::time::sleep(Duration::from_millis(1000)).await;
        perft.stop().await;

        println!("count: {}", perft.count().to_formatted_string(&Locale::en));
        assert!(perft.count() > 100_000);
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
