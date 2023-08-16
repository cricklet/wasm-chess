use std::collections::HashSet;

use lazy_static::lazy_static;
use strum::IntoEnumIterator;

use super::{
    bitboard::{single_bitboard, Bitboard, BoardIndex, FileRank},
    game::Game,
    types::{Piece, Player},
};

lazy_static! {
    static ref ROOK_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player([
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 2, 2, 2, 2, 2, 2, 0],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [-1, 0, 0, 0, 0, 0, 0, -1],
            [-1, 0, 0, 0, 0, 0, 0, -1],
            [-1, 0, 0, 0, 0, 0, 0, -1],
            [-1, 0, 0, 1, 1, 0, 0, -1],
            [0, 0, 1, 2, 2, 1, 0, 0],
        ]);
    static ref PAWN_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player([
            [4, 4, 4, 4, 4, 4, 4, 4],
            [3, 3, 3, 4, 4, 3, 3, 3],
            [3, 3, 3, 3, 3, 3, 3, 3],
            [2, 2, 2, 1, 1, 2, 2, 2],
            [1, 1, 1, 3, 3, 1, 1, 1],
            [0, 1, 1, 2, 2, 1, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ]);
    static ref BISHOP_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player([
            [-1, -1, -1, -1, -1, -1, -1, -1],
            [-1, 0, 0, 0, 0, 0, 0, -1],
            [-1, 0, 1, 1, 1, 1, 0, -1],
            [-1, 1, 1, 2, 2, 1, 1, -1],
            [-1, 0, 1, 2, 2, 1, 0, -1],
            [-1, 2, 2, 2, 2, 2, 2, -1],
            [-1, 1, 0, 0, 0, 0, 1, -1],
            [-1, -1, -1, -1, -1, -1, -1, -1],
        ]);
    static ref KNIGHT_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player([
            [-2, -2, -2, -2, -2, -2, -2, -2],
            [-2, -1, 0, 0, 0, 0, -1, -2],
            [-2, 0, 1, 2, 2, 1, 0, -2],
            [-2, 1, 2, 2, 2, 2, 1, -2],
            [-2, 0, 2, 2, 2, 2, 0, -2],
            [-2, 1, 1, 2, 2, 1, 1, -2],
            [-2, -1, 0, 0, 0, 0, -1, -2],
            [-2, -2, -2, -2, -2, -2, -2, -2],
        ]);
    static ref QUEEN_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player([
            [-1, -1, -1, -1, -1, -1, -1, -1],
            [-1, 1, 1, 1, 1, 1, 1, -1],
            [-1, 0, 0, 0, 0, 0, 0, -1],
            [-1, 0, 0, 0, 0, 0, 0, -1],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [-1, 0, 0, 0, 0, 0, 0, -1],
            [-1, 0, 0, 1, 1, 0, 0, -1],
            [-1, -1, -1, 0, 0, -1, -1, -1],
        ]);
    static ref ENEMY_KING_ENDGAME_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player([
            [4, 3, 2, 2, 2, 2, 4, 4],
            [3, 2, 1, 1, 1, 1, 2, 3],
            [2, 1, 0, 0, 0, 0, 1, 2],
            [2, 1, 0, 0, 0, 0, 1, 2],
            [2, 1, 0, 0, 0, 0, 1, 2],
            [2, 1, 0, 0, 0, 0, 1, 2],
            [3, 2, 1, 1, 1, 1, 2, 3],
            [4, 3, 2, 2, 2, 2, 4, 4],
        ]);
}

fn evaluation_bitboards_from_point_board(point_board: [[isize; 8]; 8]) -> Vec<(isize, Bitboard)> {
    let mut bitboards = Vec::new();
    let point_values = HashSet::<isize>::from_iter(point_board.iter().flatten().map(|&x| x));

    for point_value in point_values {
        let mut bitboard = Bitboard::default();
        for index in 0..64 {
            let FileRank { file, rank } = FileRank::from_index(index);
            if point_board[7 - rank][file] == point_value {
                bitboard |= single_bitboard(BoardIndex::from(index));
            }
        }
        bitboards.push((point_value, bitboard));
    }

    bitboards
}

fn evaluation_bitboards_per_player(point_board: [[isize; 8]; 8]) -> [Vec<(isize, Bitboard)>; 2] {
    let mut flipped_point_board = [[0; 8]; 8];
    for rank in 0..8 {
        for file in 0..8 {
            flipped_point_board[rank][file] = point_board[7 - rank][file];
        }
    }

    [
        evaluation_bitboards_from_point_board(point_board),
        evaluation_bitboards_from_point_board(flipped_point_board),
    ]
}

fn evaluation_bitboards_for_piece(player: Player, piece: Piece) -> &'static Vec<(isize, Bitboard)> {
    match piece {
        Piece::Pawn => &PAWN_DEVELOPMENT_BBS[player as usize],
        Piece::Rook => &ROOK_DEVELOPMENT_BBS[player as usize],
        Piece::Bishop => &BISHOP_DEVELOPMENT_BBS[player as usize],
        Piece::Knight => &KNIGHT_DEVELOPMENT_BBS[player as usize],
        Piece::Queen => &QUEEN_DEVELOPMENT_BBS[player as usize],
        Piece::King => &ENEMY_KING_ENDGAME_BBS[player as usize],
    }
}
fn centipawn_evaluation(game: &Game, player: Player) -> isize {
    let mut score = 0;

    {
        let pieces = &game.bitboards().pieces[player];
        for piece in Piece::iter() {
            score += pieces[piece].count_ones() as isize * piece.centipawns();
        }
    }

    {
        let enemy = player.other();
        let pieces = &game.bitboards().pieces[enemy];
        for piece in Piece::iter() {
            score -= pieces[piece].count_ones() as isize * piece.centipawns();
        }
    }

    score
}

fn development_evaluation(game: &Game, player: Player) -> isize {
    let mut score = 0;

    let enemy = player.other();

    let player_pieces = game.bitboards().pieces[player];
    let enemy_pieces = game.bitboards().pieces[enemy];
    for piece in Piece::iter() {
        {
            let player_piece = player_pieces[piece];
            for (score_multiple, score_bitboard) in evaluation_bitboards_for_piece(player, piece) {
                score += score_multiple * (player_piece & score_bitboard).count_ones() as isize;
            }
        }

        {
            let enemy_piece = enemy_pieces[piece];
            for (score_multiple, score_bitboard) in evaluation_bitboards_for_piece(enemy, piece) {
                score -= score_multiple * (enemy_piece & score_bitboard).count_ones() as isize;
            }
        }
    }

    score
}

pub fn evaluate(game: &Game) -> isize {
    centipawn_evaluation(game, game.player()) + development_evaluation(game, game.player())
}

#[test]
fn test_early_game_evaluation() {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    assert_eq!(development_evaluation(&game, game.player()), 0);

    // after e4, white is winning
    let game =
        Game::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap();
    let score = development_evaluation(&game, game.player());
    assert!(score < 0, "{} should be negative", score);

    // after e6, white is still winning
    let game =
        Game::from_fen("rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2").unwrap();
    let score = development_evaluation(&game, game.player());
    assert!(score > 0, "{} should be positive", score);
}

#[test]
fn test_point_evaluation() {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    assert_eq!(centipawn_evaluation(&game, game.player()), 0);

    let game =
        Game::from_fen("rnbqkbnr/ppp2ppp/4p3/3P4/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 3").unwrap();
    let score = centipawn_evaluation(&game, game.player());
    assert_eq!(score, -100, "white has taken a pawn");

    let game =
        Game::from_fen("rnbqkbnr/ppp2ppp/4p3/3P4/3P4/8/PPP2PPP/RNBQKBNR w KQkq - 0 3").unwrap();
    let score = centipawn_evaluation(&game, game.player());
    assert_eq!(score, 100, "white has taken a pawn");
}
