use std::{future::Future, sync::Mutex, pin::Pin};

use crate::{game::{Game, Legal}, perft::{PerftLoop, PerftLoopResult}};

use super::iterative_traversal::TraversalStack;

type YieldFn = fn() -> Pin<Box<dyn Future<Output = ()> + Send>>;
type LogFn = fn(&str);
type DoneFn = fn(usize);

#[derive(Debug)]
pub struct AsyncPerftRunner
{
    data: Mutex<Option<PerftLoop>>,

    stop: Mutex<bool>,

    yield_fn: YieldFn,
    log_fn: LogFn,
    done_fn: DoneFn,
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
            *data = Some(PerftLoop::new(&fen, max_depth));
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
                    PerftLoopResult::Interrupted
                } else {
                    data.iterate_loop()
                }
            };

            match result {
                PerftLoopResult::Continue => {
                    (self.yield_fn)().await;
                }
                PerftLoopResult::Done => {
                    break;
                }
                PerftLoopResult::Interrupted => {
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
