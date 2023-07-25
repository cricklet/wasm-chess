#![allow(dead_code)]
#![allow(unused_imports)]

pub mod bitboards;
pub mod game;
pub mod helpers;
pub mod moves;
pub mod perft;
pub mod types;

fn main() {
    let game: game::Game = game::Game::new();
    println!("Hello, {:?}", game);
}
