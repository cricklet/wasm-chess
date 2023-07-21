use std::iter;

use crate::{
    game::Game,
    helpers::ErrorResult,
    moves::{all_moves, index_in_danger, OnlyCaptures, OnlyQueenPromotion},
    types::Piece,
};

fn assert_fen_matches(expected_fen: &str) {
    let game = Game::from_fen(expected_fen).unwrap();
    let game_fen = game.to_fen();

    let expected_fen: Vec<&str> = expected_fen.split(" ").collect();
    let game_fen: Vec<&str> = game_fen.split(" ").collect();

    for (expected, actual) in expected_fen.iter().zip(game_fen.iter()) {
        assert_eq!(expected, actual);
    }
}

#[test]
fn test_fen_start_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
    assert_fen_matches(fen);
}

// fn traverse_game_iter(game: &Game, depth: u8, max_depth: u8) -> GameTraversal {
//     if depth >= max_depth {
//         // return Ok(Box::new(iter::empty()));
//         todo!()
//     }

//     let moves =
//         all_moves(game.player, &game, OnlyCaptures::NO, OnlyQueenPromotion::NO).map(|m| m.unwrap());
//     let future = moves.map(move |m| game.make_move(m)).map(|g| g.unwrap());
//     let legal_futures = future.filter(|next_game| {
//         let king_index = next_game.board.index_of_piece(game.player, Piece::King);
//         let illegal_move = index_in_danger(game.player, king_index, next_game).unwrap();
//         !illegal_move
//     });
//     let legal_futures =
//         legal_futures.map(move |next_game| traverse_game_iter(&next_game, depth + 1, max_depth));

//     let traversal = GameTraversal {
//         game,
//         future: Box::new(legal_futures),
//     };

//     traversal
// }

fn traverse_game(game: &Game, depth: u8, max_depth: u8) -> usize {
    if depth >= max_depth {
        return 1;
    }

    let mut result = 0;

    let player = game.player;

    let moves = all_moves(player, game, OnlyCaptures::NO, OnlyQueenPromotion::NO);
    for m in moves {
        let m = m.unwrap();
        let ref next_game = game.make_move(m);
        let next_game = next_game.as_ref().unwrap();

        let king_index = next_game.board.index_of_piece(player, Piece::King);
        let illegal_move = index_in_danger(player, king_index, next_game).unwrap();

        if illegal_move {
            continue;
        }

        result += traverse_game(next_game, depth + 1, max_depth);
    }

    result
}

fn assert_perft_matches(fen: &str, expected_counts: &[u64]) {
    let game = Game::from_fen(fen).unwrap();

    assert_eq!(game.to_fen(), fen);

    traverse_game(&game, 0, 2);

    // let mut perft = perft::Perft::new(game);
    // for (depth, expected_count) in expected_counts.iter().enumerate() {
    //     let count = perft.perft(depth as u8);
    //     assert_eq!(count, *expected_count);
    // }
}

#[test]
fn test_perft_start_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let expected_count = [
        1, 20, 400, 8902, 197281,
        // 4865609,
        // 119060324,
        // 3195901860,
    ];
    assert_perft_matches(fen, &expected_count);
}

// struct GameTraversal {
//     current: Game,
//     future: Box<dyn Iterator<Item = GameTraversal>>,
// }

// fn game_traversal_iter(traversal: &GameTraversal) -> Box<dyn Iterator<Item = &Game>> {
//     let once = iter::once(&traversal.current);
//     let future = traversal.future.map(|t| game_traversal_iter(&t));
//     let future = future.flatten();

//     Box::new(once)
// }

struct InfiniteStrings {
    current: String,
    next: Box<dyn Iterator<Item = InfiniteStrings>>,
}

impl InfiniteStrings {
    fn new() -> InfiniteStrings {
        InfiniteStrings {
            current: "x".to_string(),
            next: Box::new(std::iter::empty()),
        }
    }

    fn traverse(self) -> Box<dyn Iterator<Item = String>> {
        let once = std::iter::once(self.current);
        let future = self.next.map(|s| s.traverse());
        let future = future.flatten();

        let all = once.chain(future);

        Box::new(all)
    }
}

#[test]
fn test_understand_traversal_iter_string() {
    let infinite = InfiniteStrings::new();
    for s in infinite.traverse() {
        println!("{}", s);
    }
}
