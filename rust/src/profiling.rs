use std::{backtrace::Backtrace, fs::File, io::Write, iter};

use pprof::protos::Message;

use crate::{game::Game, perft::run_perft_iteratively};

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

impl<'a> Drop for Profiler<'a> {
    fn drop(&mut self) {
        drop(&mut self.guard)
    }
}

#[test]
fn test_perft_start_board_iteratively() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let expected_count = [
        1, 20, 400, 8902, 197281, 4865609,
        // 119060324,
        // 3195901860,
    ];

    run_perft_iteratively(Game::from_fen(fen).unwrap(), 2, 1000).unwrap();
    {
        let p = Profiler::new("iterative_perft".to_string());
        for (i, expected_count) in
            expected_count.into_iter().enumerate().collect::<Vec<_>>()[2..].into_iter()
        {
            let max_depth = i + 1;
            let max_iterations = max_depth * expected_count;

            let count =
                run_perft_iteratively(Game::from_fen(fen).unwrap(), max_depth, max_iterations)
                    .unwrap();
            assert_eq!(count, *expected_count);
        }
        p.flush();
    }
}
