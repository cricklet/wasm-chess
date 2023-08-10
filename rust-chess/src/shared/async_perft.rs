use std::{future::Future, sync::Mutex};

use crate::game::{Game, Legal};

use super::iterative_traversal::TraversalStack;

#[derive(Debug)]
struct AsyncPerftData {
    stack: TraversalStack<MAX_PERFT_DEPTH>,
    count: usize,

    max_depth: usize,
    start_fen: String,
}

impl AsyncPerftData {
    pub fn new(fen: &str, max_depth: usize) -> Self {
        if max_depth > MAX_PERFT_DEPTH {
            panic!("max_depth must be <= {}", MAX_PERFT_DEPTH);
        }

        let game = Game::from_fen(fen).unwrap();
        let stack = TraversalStack::<MAX_PERFT_DEPTH>::new(game).unwrap();

        Self {
            stack,
            count: 0,
            max_depth,
            start_fen: fen.to_string(),
        }
    }

    fn iterate(&mut self) -> AsyncPerftIterationResult {
        let ref mut traversal = self.stack;

        // Leaf node case:
        if traversal.depth + 1 >= self.max_depth {
            self.count += 1;
            traversal.depth -= 1;

            return AsyncPerftIterationResult::Continue;
        }

        // We have moves to traverse, dig deeper
        let next_move = traversal.next_move().unwrap();
        if let Some(next_move) = next_move {
            let (current, next) = traversal.current_and_next_mut().unwrap();

            let result = next.setup_from_move(current, &next_move).unwrap();
            if result == Legal::No {
                return AsyncPerftIterationResult::Continue;
            } else {
                traversal.depth += 1;
                return AsyncPerftIterationResult::Continue;
            }
        }

        // We're out of moves to traverse, pop back up.
        if traversal.depth == 0 {
            return AsyncPerftIterationResult::Done;
        } else {
            traversal.depth -= 1;
            return AsyncPerftIterationResult::Continue;
        }
    }

    pub fn iterate_loop(&mut self) -> AsyncPerftIterationResult {
        for _ in 0..100_000 {
            let result = self.iterate();
            if result != AsyncPerftIterationResult::Continue {
                return result;
            }
        }

        AsyncPerftIterationResult::Continue
    }


}

#[derive(Debug)]
pub struct AsyncPerft<C, F>
where
    C: Fn(AsyncPerftMessage) -> F,
    F: Future<Output = ()>,
{
    data: Mutex<Option<AsyncPerftData>>,

    stop: Mutex<bool>,
    js_callback: C,
}

const MAX_PERFT_DEPTH: usize = 10;
pub enum AsyncPerftMessage {
    Count(usize),
    Log(String),
    Continue,
}

#[derive(Debug, PartialEq, Eq)]
enum AsyncPerftIterationResult {
    Continue,
    Done,
    Interrupted,
}

impl<C, F> AsyncPerft<C, F>
where
    C: Fn(AsyncPerftMessage) -> F,
    F: Future<Output = ()>,
{
    pub fn new(js_callback: C) -> Self {
        Self {
            data: Mutex::new(None),
            stop: Mutex::new(false),
            js_callback,
        }
    }

    pub fn count(&self) -> isize {
        let data = self.data.lock();
        let data = data.unwrap();

        match data.as_ref() {
            Some(data) => data.count as isize,
            None => -1,
        }
    }
    pub async fn start(&self, fen: String, max_depth: usize) {
        println!("starting");

        {
            let mut data = self.data.lock().unwrap();
            *data = Some(AsyncPerftData::new(&fen, max_depth));
        }

        loop {
            let result = {
                let mut data = self.data.lock().unwrap();
                let data = data.as_mut().unwrap();

                if (data.start_fen != fen) || (data.max_depth != max_depth) {
                    AsyncPerftIterationResult::Interrupted
                } else {
                    data.iterate_loop()
                }
            };

            match result {
                AsyncPerftIterationResult::Continue => {
                    (self.js_callback)(AsyncPerftMessage::Continue).await;
                }
                AsyncPerftIterationResult::Done => {
                    break;
                }
                AsyncPerftIterationResult::Interrupted => {
                    panic!("interrupted: please only one perft running at a time");
                }
            }
        }

        println!("start done");
    }

    pub async fn stop(&self) {
        println!("stopping");
        *self.stop.lock().unwrap() = true;
        println!("stopped");
    }
}

#[cfg(test)]
mod test {

    use num_format::{ToFormattedString, Locale};

    use super::*;
    use std::{sync::Arc, time::Duration};

    #[tokio::test()]
    async fn test_async_perft_short() {
        use crate::game::Game;

        let perft = AsyncPerft::new(|_| async {
            tokio::time::sleep(Duration::from_millis(1)).await;
        });
        perft.start("startpos".to_string(), 2).await;
        perft.stop().await;
        assert_eq!(perft.count(), 20);
    }

    #[tokio::test()]
    async fn test_async_perft_long() {
        use crate::game::Game;

        let perft = Arc::new(AsyncPerft::new(|message| async {
            match message {
                AsyncPerftMessage::Count(count) => {
                    println!("count: {}", count);
                }
                AsyncPerftMessage::Log(s) => {
                    println!("{}", s);
                }
                AsyncPerftMessage::Continue => {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
        }));

        let spawn_perft = perft.clone();
        tokio::spawn(async move { spawn_perft.start("startpos".to_string(), 7).await });

        tokio::time::sleep(Duration::from_millis(200)).await;
        perft.stop().await;

        println!("count: {}", perft.count().to_formatted_string(&Locale::en));
        assert!(perft.count() > 100_000);
    }
}
