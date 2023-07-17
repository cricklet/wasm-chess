use crate::{
    game::Game,
    moves::{all_moves, OnlyCaptures},
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

fn traverse_game(game: &Game, depth: u8) {
    if depth == 0 {
        return;
    }

    println!("depth: {}, game: {}", depth, game.pretty());

    let moves = all_moves(game.player, game, OnlyCaptures::NO);
    for m in moves {
        match m {
            Ok(m) => {
                let ref next_game = game.make_move(m);
                match next_game {
                    Ok(next_game) => {
                        traverse_game(next_game, depth - 1);
                    }
                    Err(e) => panic!("error: {}", e),
                }
            }
            Err(e) => panic!("error: {}", e),
        }
    }
}

fn assert_perft_matches(fen: &str, expected_counts: &[u64]) {
    let game = Game::from_fen(fen).unwrap();

    assert_eq!(game.to_fen(), fen);

    println!("game: {}", game.pretty());

    traverse_game(&game, 1);

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
