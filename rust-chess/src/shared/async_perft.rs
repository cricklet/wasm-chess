#[derive(Debug)]
struct AsyncPerftData {
    stack: TraversalStack<MAX_PERFT_DEPTH>,
    count: usize,
}

#[derive(Debug)]
pub struct AsyncPerft {
    data: Mutex<Option<AsyncPerftData>>,
    stop: Mutex<bool>,

    yield_callback: fn(&AsyncPerftMessage),
}

const MAX_PERFT_DEPTH: usize = 10;
pub enum AsyncPerftMessage {
    Count(usize),
    Log(String),
}

impl AsyncPerft {
    pub fn new(callback: fn(&AsyncPerftMessage)) -> Self {
        set_panic_hook();
        Self {
            data: Mutex::new(None),
            stop: Mutex::new(false),
            yield_callback: callback,
        }
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
        log_to_js(&"perft start");
        yield_to_js().await;

        if max_depth > MAX_PERFT_DEPTH {
            panic!("max_depth must be <= {}", MAX_PERFT_DEPTH);
        }

        loop {
            log_to_js("perft loop");
            yield_to_js().await;

            if *self.stop.lock().unwrap() {
                return;
            }

            self.lazy_init(&fen);

            {
                let mut data = self.data.lock().unwrap();
                let data = data.as_mut().unwrap();
                let ref mut traversal = data.stack;

                log_to_js(&format!("{:#?}", traversal.current()));
                yield_to_js().await;

                // Leaf node case:
                if traversal.depth + 1 >= max_depth {
                    data.count += 1;
                    traversal.depth -= 1;

                    yield_to_js().await;
                }

                // We have moves to traverse, dig deeper
                let next_move = traversal.next_move().unwrap();
                if let Some(next_move) = next_move {
                    let (current, next) = traversal.current_and_next_mut().unwrap();

                    let result = next.setup_from_move(current, &next_move).unwrap();
                    if result == Legal::No {
                        yield_to_js().await;
                        continue;
                    } else {
                        traversal.depth += 1;
                        yield_to_js().await;
                        continue;
                    }
                }

                // We're out of moves to traverse, pop back up.
                if traversal.depth == 0 {
                    break;
                } else {
                    traversal.depth -= 1;
                    yield_to_js().await;
                    continue;
                }
            }
        }
    }

    pub fn stop(&self) {
        log_to_js(&"perft stop");
        *self.stop.lock().unwrap() = true;
    }
}
