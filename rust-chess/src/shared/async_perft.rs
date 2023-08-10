use std::{future::Future, sync::Mutex};

use crate::game::{Game, Legal};

use super::iterative_traversal::TraversalStack;

#[derive(Debug)]
struct AsyncPerftData {
    stack: TraversalStack<MAX_PERFT_DEPTH>,
    count: usize,
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
    Yield,
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

    async fn log(&self, s: &str) {
        (self.js_callback)(AsyncPerftMessage::Log(s.to_string())).await;
    }

    async fn yield_to_js(&self) {
        (self.js_callback)(AsyncPerftMessage::Yield).await;
    }

    pub fn count(&self) -> usize {
        let data = self.data.lock();
        let data = data.unwrap();
        let data = data.as_ref().unwrap();
        data.count
    }

    fn lazy_init(&self, fen: &str) {
        let mut data = self.data.lock().unwrap();
        if data.is_none() {
            let game = Game::from_fen(&fen).unwrap();
            let stack = TraversalStack::<MAX_PERFT_DEPTH>::new(game).unwrap();
            *data = Some(AsyncPerftData { stack, count: 0 });
        }
    }

    pub async fn start(&self, fen: String, max_depth: usize) {
        self.log("perft start").await;

        if max_depth > MAX_PERFT_DEPTH {
            panic!("max_depth must be <= {}", MAX_PERFT_DEPTH);
        }

        loop {
            self.log("perft loop").await;

            if *self.stop.lock().unwrap() {
                break;
            }

            self.lazy_init(&fen);

            {
                let mut data = self.data.lock().unwrap();
                let data = data.as_mut().unwrap();
                let ref mut traversal = data.stack;

                self.log(&format!("{:#?}", traversal.current())).await;

                // Leaf node case:
                if traversal.depth + 1 >= max_depth {
                    data.count += 1;
                    traversal.depth -= 1;

                    self.yield_to_js().await;
                }

                // We have moves to traverse, dig deeper
                let next_move = traversal.next_move().unwrap();
                if let Some(next_move) = next_move {
                    let (current, next) = traversal.current_and_next_mut().unwrap();

                    let result = next.setup_from_move(current, &next_move).unwrap();
                    if result == Legal::No {
                        self.yield_to_js().await;
                        continue;
                    } else {
                        traversal.depth += 1;
                        self.yield_to_js().await;
                        continue;
                    }
                }

                // We're out of moves to traverse, pop back up.
                if traversal.depth == 0 {
                    break;
                } else {
                    traversal.depth -= 1;
                    self.yield_to_js().await;
                    continue;
                }
            }
        }

        self.log("perft done").await;
    }

    pub async fn stop(&self) {
        self.log("perft stopping").await;
        *self.stop.lock().unwrap() = true;
        self.log("perft stopped").await;
    }
}
