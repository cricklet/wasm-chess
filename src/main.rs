#![allow(dead_code)]
pub mod bitboards;
pub mod game;
pub mod helpers;
pub mod types;

fn main() {
    let game: game::Game = game::Game::new();
    println!("Hello, {:?}", game);
}
