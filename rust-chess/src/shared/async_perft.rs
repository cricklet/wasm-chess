use std::{future::Future, sync::Mutex, pin::Pin};

use crate::game::{Game, Legal};

use super::iterative_traversal::TraversalStack;

#[derive(Debug)]
struct AsyncPerftData {
    stack: TraversalStack<MAX_PERFT_DEPTH>,
    count: usize,

    max_depth: usize,
    start_fen: String,

    loop_count: usize,
}
const LOOP_COUNT: usize = 1_000_000;

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
            loop_count: LOOP_COUNT,
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
        println!("iterating loop");
        for _ in 0..self.loop_count {
            let result = self.iterate();
            if result != AsyncPerftIterationResult::Continue {
                return result;
            }
        }

        AsyncPerftIterationResult::Continue
    }


}

type YieldFn = fn() -> Pin<Box<dyn Future<Output = ()> + Send>>;
type LogFn = fn(&str);
type DoneFn = fn(usize);

#[derive(Debug)]
pub struct AsyncPerftRunner
{
    data: Mutex<Option<AsyncPerftData>>,

    stop: Mutex<bool>,

    yield_fn: YieldFn,
    log_fn: LogFn,
    done_fn: DoneFn,
}

const MAX_PERFT_DEPTH: usize = 10;


#[derive(Debug, PartialEq, Eq)]
pub enum AsyncPerftIterationResult {
    Continue,
    Done,
    Interrupted,
}

impl AsyncPerftRunner{
    pub fn from(yield_fn: YieldFn, log_fn: LogFn, done_fn: DoneFn) -> Self {
        Self {
            data: Mutex::new(None),
            stop: Mutex::new(false),
            yield_fn,
            log_fn,
            done_fn,
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
            {
                let stop = self.stop.lock().unwrap();
                if *stop {
                    break;
                }
            }
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
                    (self.yield_fn)().await;
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
        (self.done_fn)(self.count() as usize);
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

    use crate::game::Game;
    use super::*;
    use std::{sync::Arc, time::Duration};

    fn log_fn(s: &str) {
        println!("{}", s);
    }

    fn yield_fn() -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(tokio::time::sleep(Duration::from_millis(1)))
    }

    fn done_fn(count: usize) {
        println!("finished! {}", count);
    }

    #[tokio::test()]
    async fn test_async_perft_short() {
        let perft = AsyncPerftRunner::from(
            yield_fn,
            log_fn,
            done_fn,
        );
        perft.start("startpos".to_string(), 2).await;
        perft.stop().await;
        assert_eq!(perft.count(), 20);
    }

    #[tokio::test()]
    async fn test_async_perft_short_spawn() {
        let perft = Arc::new(AsyncPerftRunner::from(
            yield_fn,
            log_fn,
            done_fn,
        ));

        let spawn_perft = perft.clone();
        tokio::spawn(async move { spawn_perft.start("startpos".to_string(), 2).await });

        tokio::time::sleep(Duration::from_millis(200)).await;
        perft.stop().await;

        println!("count: {}", perft.count().to_formatted_string(&Locale::en));
        assert_eq!(perft.count(), 20);
    }

    #[tokio::test()]
    async fn test_async_perft_long() {
        let perft = Arc::new(AsyncPerftRunner::from(
            yield_fn,
            log_fn,
            done_fn,
        ));

        let spawn_perft = perft.clone();
        let spawn_handle = tokio::spawn(async move { spawn_perft.start("startpos".to_string(), 7).await });

        tokio::time::sleep(Duration::from_millis(1000)).await;
        perft.stop().await;

        spawn_handle.await.unwrap();

        println!("count: {}", perft.count().to_formatted_string(&Locale::en));
        assert!(perft.count() > 50_000);
    }
}
