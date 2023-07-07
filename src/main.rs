
pub mod game;
pub mod bitboards;


fn main() {
    let game: game::Game = game::Game::new();
    println!("Hello, {:?}", game);
}
