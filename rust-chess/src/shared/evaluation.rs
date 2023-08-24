use std::{
    cmp::{max, min},
    collections::HashSet,
};

use lazy_static::lazy_static;
use strum::IntoEnumIterator;

use crate::bitboard::{bitboard_from_string, ForPlayer};

use super::{
    bitboard::{single_bitboard, Bitboard, BoardIndex, FileRank},
    game::Game,
    types::{Piece, Player},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum GameStage {
    Early,
    Late,
}

lazy_static! {
    static ref EARLY_ROOK_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player(
            10,
            [
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 1, 1, 0, 0, 0],
                [0, 0, 0, 1, 1, 0, 0, 0],
            ]
        );
    static ref EARLY_PAWN_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player(
            15,
            [
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 1, 3, 3, 1, 0, 0],
                [0, 1, 0, 2, 2, 0, 1, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
            ]
        );
    static ref LATE_PAWN_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player(
            10,
            [
                [4, 4, 4, 4, 4, 4, 4, 4],
                [4, 4, 4, 4, 4, 4, 4, 4],
                [3, 3, 3, 3, 3, 3, 3, 3],
                [2, 2, 2, 2, 2, 2, 2, 2],
                [1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 0, 0, 0, 0, 1, 1],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
            ]
        );
    static ref EARLY_BISHOP_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player(
            10,
            [
                [-1, -1, -1, -1, -1, -1, -1, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 1, 1, 1, 1, 0, -1],
                [-1, 1, 1, 1, 1, 1, 1, -1],
                [-1, 1, 0, 1, 1, 0, 1, -1],
                [-1, -1, -1, -1, -1, -1, -1, -1],
            ]
        );
    static ref EARLY_KNIGHT_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player(
            10,
            [
                [-1, -1, -1, -1, -1, -1, -1, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 1, 1, 1, 1, 0, -1],
                [-1, 1, 1, 1, 1, 1, 1, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, -1, -1, -1, -1, -1, -1, -1],
            ]
        );
    static ref EARLY_QUEEN_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player(
            10,
            [
                [-1, -1, -1, -1, -1, -1, -1, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 0, -1, -1, 0, 0, -1],
                [0, 0, 0, -1, -1, 0, 0, 0],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, 0, 0, 1, 1, 0, 0, -1],
                [-1, -1, -1, 0, 0, -1, -1, -1],
            ]
        );
    static ref EARLY_KING_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player(
            10,
            [
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 1, 0, 0, 0, 1, 0],
            ]
        );
    static ref LATE_KING_DEVELOPMENT_BBS: [Vec<(isize, Bitboard)>; 2] =
        evaluation_bitboards_per_player(
            20,
            [
                [-1, -1, 0, 0, 0, 0, -1, -1],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [0, 0, 1, 1, 1, 1, 0, 0],
                [0, 0, 1, 1, 1, 1, 0, 0],
                [0, 0, 1, 1, 1, 1, 0, 0],
                [0, 0, 1, 1, 1, 1, 0, 0],
                [-1, 0, 0, 0, 0, 0, 0, -1],
                [-1, -1, 0, 0, 0, 0, -1, -1],
            ]
        );
}

fn evaluation_bitboards_from_point_board(
    scale: isize,
    point_board: [[isize; 8]; 8],
) -> Vec<(isize, Bitboard)> {
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
        bitboards.push((scale * point_value, bitboard));
    }

    bitboards
}

fn evaluation_bitboards_per_player(
    scale: isize,
    point_board: [[isize; 8]; 8],
) -> [Vec<(isize, Bitboard)>; 2] {
    let mut flipped_point_board = [[0; 8]; 8];
    for rank in 0..8 {
        for file in 0..8 {
            flipped_point_board[rank][file] = point_board[7 - rank][file];
        }
    }

    [
        evaluation_bitboards_from_point_board(scale, point_board),
        evaluation_bitboards_from_point_board(scale, flipped_point_board),
    ]
}

fn evaluation_bitboards_for(
    player: Player,
    development_bbs: &(isize, [Vec<(isize, Bitboard)>; 2]),
) -> (isize, &Vec<(isize, Bitboard)>) {
    (
        development_bbs.0,
        development_bbs.1.get(player as usize).unwrap(),
    )
}

fn evaluation_bitboards_for_piece(
    stage: GameStage,
    player: Player,
    piece: Piece,
) -> Option<&'static Vec<(isize, Bitboard)>> {
    match stage {
        GameStage::Early => match piece {
            Piece::Pawn => Some(&EARLY_PAWN_DEVELOPMENT_BBS[player as usize]),
            Piece::Rook => Some(&EARLY_ROOK_DEVELOPMENT_BBS[player as usize]),
            Piece::Bishop => Some(&EARLY_BISHOP_DEVELOPMENT_BBS[player as usize]),
            Piece::Knight => Some(&EARLY_KNIGHT_DEVELOPMENT_BBS[player as usize]),
            Piece::Queen => Some(&EARLY_QUEEN_DEVELOPMENT_BBS[player as usize]),
            Piece::King => Some(&EARLY_KING_DEVELOPMENT_BBS[player as usize]),
        },
        GameStage::Late => match piece {
            Piece::Pawn => Some(&LATE_PAWN_DEVELOPMENT_BBS[player as usize]),
            Piece::King => Some(&LATE_KING_DEVELOPMENT_BBS[player as usize]),
            _ => None,
        },
    }
}

fn development_evaluation(stage: GameStage, game: &Game, player: Player) -> isize {
    let mut score = 0;

    let enemy = player.other();

    let player_pieces = game.bitboards().pieces[player];
    let enemy_pieces = game.bitboards().pieces[enemy];

    for piece in Piece::iter() {
        {
            let player_piece = player_pieces[piece];
            if let Some(eval_bbs) = evaluation_bitboards_for_piece(stage, player, piece) {
                for (score_multiple, score_bitboard) in eval_bbs {
                    score += score_multiple * (player_piece & score_bitboard).count_ones() as isize;
                }
            }
        }

        {
            let enemy_piece = enemy_pieces[piece];
            if let Some(eval_bbs) = evaluation_bitboards_for_piece(stage, enemy, piece) {
                for (score_multiple, score_bitboard) in eval_bbs {
                    score -= score_multiple * (enemy_piece & score_bitboard).count_ones() as isize;
                }
            }
        }
    }

    score
}

lazy_static! {
    static ref STARTING_CENTIPAWNS: isize = {
        let mut score = 0;
        score += Piece::Pawn.centipawns() * 8;
        score += Piece::Knight.centipawns() * 2;
        score += Piece::Bishop.centipawns() * 2;
        score += Piece::Rook.centipawns() * 2;
        score += Piece::Queen.centipawns() * 1;
        score
    };
}
fn centipawns_for_player(player: Player, game: &Game) -> isize {
    let mut score = 0;
    let pieces = &game.bitboards().pieces[player];
    for piece in Piece::iter() {
        score += pieces[piece].count_ones() as isize * piece.centipawns();
    }
    score
}

fn pieces_missing_for_player(player: Player, game: &Game) -> usize {
    let mut missing: usize = 0;
    let pieces = &game.bitboards().pieces[player];
    missing += 8 - min(8, pieces[Piece::Pawn].count_zeros()) as usize;
    missing += 2 - min(2, pieces[Piece::Rook].count_zeros()) as usize;
    missing += 2 - min(2, pieces[Piece::Bishop].count_zeros()) as usize;
    missing += 2 - min(2, pieces[Piece::Knight].count_zeros()) as usize;
    missing += 1 - min(1, pieces[Piece::Queen].count_zeros()) as usize;
    missing
}

fn centipawn_evaluation(player: Player, game: &Game) -> isize {
    let player_centipawns = centipawns_for_player(player, game);
    let enemy_centipawns = centipawns_for_player(player.other(), game);
    player_centipawns - enemy_centipawns
}

fn bitboard_and_flipped(bb: Bitboard) -> ForPlayer<Bitboard> {
    let mut flipped = Bitboard::default();
    for rank in 0..8 {
        for file in 0..8 {
            if bb & single_bitboard(BoardIndex::from_file_rank(file, rank)) != 0 {
                flipped |= single_bitboard(BoardIndex::from_file_rank(file, 7 - rank));
            }
        }
    }
    ForPlayer {
        white: bb,
        black: flipped,
    }
}

lazy_static! {
    static ref E_FILE_PAWN: ForPlayer<Bitboard> = bitboard_and_flipped(
        single_bitboard(BoardIndex::from_str("e2").unwrap())
            | single_bitboard(BoardIndex::from_str("e3").unwrap())
            | single_bitboard(BoardIndex::from_str("e4").unwrap())
    );
    static ref D_FILE_PAWN: ForPlayer<Bitboard> = bitboard_and_flipped(
        single_bitboard(BoardIndex::from_str("d2").unwrap())
            | single_bitboard(BoardIndex::from_str("d3").unwrap())
            | single_bitboard(BoardIndex::from_str("d4").unwrap())
    );
}

fn keep_center_pawns(player: Player, game: &Game) -> isize {
    let pawns = game.bitboards().pieces[player][Piece::Pawn];
    let has_e = pawns & E_FILE_PAWN[player] != 0;
    let has_d = pawns & D_FILE_PAWN[player] != 0;
    if has_e && has_d {
        10
    } else {
        0
    }
}

#[test]
fn test_keep_center_pawns() {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    assert!(keep_center_pawns(Player::White, &game) > 0);
    assert!(keep_center_pawns(Player::Black, &game) > 0);

    let game =
        Game::from_fen("rnbqkbnr/pppp1ppp/4p3/3P4/3P4/8/PPP2PPP/RNBQKBNR w KQkq - 0 3").unwrap();
    assert!(keep_center_pawns(Player::White, &game) == 0);
    assert!(keep_center_pawns(Player::Black, &game) > 0);

    let game =
        Game::from_fen("rnbqkbnr/pppp1ppp/8/3p4/3P4/8/PPP2PPP/RNBQKBNR w KQkq - 0 4").unwrap();
    assert!(keep_center_pawns(Player::White, &game) == 0);
    assert!(keep_center_pawns(Player::Black, &game) == 0);
}

pub fn evaluate(game: &Game) -> isize {
    let player = game.player();
    let enemy = player.other();

    let player_centipawns = centipawns_for_player(player, game);
    let enemy_centipawns = centipawns_for_player(player.other(), game);

    let stage = if player_centipawns < *STARTING_CENTIPAWNS - 800
        || enemy_centipawns < *STARTING_CENTIPAWNS - 800
        || pieces_missing_for_player(player, game) + pieces_missing_for_player(enemy, game) >= 6
    {
        GameStage::Late
    } else {
        GameStage::Early
    };

    let mut eval =
        centipawn_evaluation(player, game) + development_evaluation(stage, game, game.player());

    if stage == GameStage::Early {
        eval += keep_center_pawns(player, game);
        eval -= keep_center_pawns(enemy, game);
    }

    eval
}

#[test]
fn test_early_game_evaluation() {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    assert_eq!(
        development_evaluation(GameStage::Early, &game, game.player()),
        0
    );

    // after e4, white is winning
    let game =
        Game::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap();
    let score = development_evaluation(GameStage::Early, &game, game.player());
    assert!(score < 0, "{} should be negative", score);

    // after e6, white is still winning
    let game =
        Game::from_fen("rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2").unwrap();
    let score = development_evaluation(GameStage::Early, &game, game.player());
    assert!(score > 0, "{} should be positive", score);
}

#[test]
fn test_point_evaluation() {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    assert_eq!(centipawn_evaluation(game.player(), &game), 0);

    let game =
        Game::from_fen("rnbqkbnr/ppp2ppp/4p3/3P4/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 3").unwrap();
    let score = centipawn_evaluation(game.player(), &game);
    assert_eq!(score, -100, "white has taken a pawn");

    let game =
        Game::from_fen("rnbqkbnr/ppp2ppp/4p3/3P4/3P4/8/PPP2PPP/RNBQKBNR w KQkq - 0 3").unwrap();
    let score = centipawn_evaluation(game.player(), &game);
    assert_eq!(score, 100, "white has taken a pawn");
}
