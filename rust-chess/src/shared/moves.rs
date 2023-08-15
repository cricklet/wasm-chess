use std::fmt::{Debug, Formatter};

use super::{bitboard::*, game::Game, helpers::*, types::*};

#[derive(Debug, Copy, Clone)]
pub struct MoveOptions {
    pub only_captures: OnlyCaptures,
    pub only_queen_promotion: OnlyQueenPromotion,
}

impl Default for MoveOptions {
    fn default() -> Self {
        Self {
            only_captures: Default::default(),
            only_queen_promotion: Default::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OnlyCaptures {
    No,
    Yes,
}

impl Default for OnlyCaptures {
    fn default() -> Self {
        Self::No
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OnlyQueenPromotion {
    No,
    Yes,
}

impl Default for OnlyQueenPromotion {
    fn default() -> Self {
        Self::No
    }
}

impl OnlyQueenPromotion {
    pub fn pieces(&self) -> &'static [Piece] {
        match self {
            OnlyQueenPromotion::No => &PROMOTION_PIECES,
            OnlyQueenPromotion::Yes => &[Piece::Queen],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Quiet {
    Castle {
        rook_start: BoardIndex,
        rook_end: BoardIndex,
    },
    PawnSkip {
        skipped_index: BoardIndex,
    },
    Move,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Capture {
    EnPassant { taken_index: BoardIndex },
    Take { taken_piece: PlayerPiece },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MoveType {
    Quiet(Quiet),
    Capture(Capture),
    Invalid,
}

#[derive(Eq, PartialEq, Clone)]
pub struct SizedMoveBuffer<const N: usize> {
    pub moves: [Move; N],
    pub size: usize,
}

pub type MoveBuffer = SizedMoveBuffer<80>;

impl<const N: usize> Debug for SizedMoveBuffer<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SizedMoveBuffer")
            .field("moves", &self.moves[..self.size].iter())
            .field("size", &self.size)
            .finish()
    }
}

impl<const N: usize> Default for SizedMoveBuffer<N> {
    fn default() -> Self {
        Self {
            moves: [Move::invalid(); N],
            size: 0,
        }
    }
}

impl<const N: usize> SizedMoveBuffer<N> {
    pub fn clear(&mut self) {
        self.size = 0;
    }

    pub fn get(&self, index: usize) -> &Move {
        &self.moves[index]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Move {
        &mut self.moves[index]
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    pub fn push_mut(&mut self) -> &mut Move {
        self.size += 1;
        let m = self.get_mut(self.size - 1);
        m
    }

    pub fn push(&mut self, m: Move) {
        self.size += 1;
        self.moves[self.size - 1] = m;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.size].iter()
    }

    pub fn collect(&self) -> Vec<Move> {
        self.moves[..self.size].to_vec()
    }
}

impl<const N: usize> IntoIterator for SizedMoveBuffer<N> {
    type Item = Move;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.moves[..self.size].to_vec().into_iter()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub piece: PlayerPiece,
    pub start_index: BoardIndex,
    pub end_index: BoardIndex,
    pub move_type: MoveType,
    pub promotion: Option<Piece>,
}

impl Default for Move {
    fn default() -> Self {
        Self::invalid()
    }
}

impl Move {
    pub fn invalid() -> Self {
        Self {
            piece: PlayerPiece::new(Player::White, Piece::Pawn),
            start_index: BoardIndex::from(0),
            end_index: BoardIndex::from(0),
            move_type: MoveType::Invalid,
            promotion: None,
        }
    }

    pub fn to_uci(&self) -> String {
        let promo = self.promotion.map(|p| p.to_uci());
        let promo = promo.unwrap_or(&"");
        format!("{}{}{}", self.start_index, self.end_index, promo)
    }

    pub fn is_quiet(&self) -> bool {
        match self.move_type {
            MoveType::Quiet(_) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {:?}", self.piece, self.to_uci(), self.move_type,)
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {:?}", self.piece, self.to_uci(), self.move_type,)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JumpingPiece {
    Knight,
    King,
}

pub fn potential_bb_to_moves(
    buffer: &mut MoveBuffer,
    PlayerPiece { player, piece }: PlayerPiece,
    piece_index: BoardIndex,
    potential: Bitboard,
    bitboards: &Bitboards,
    only_captures: OnlyCaptures,
) -> ErrorResult<()> {
    let self_occupied = bitboards.occupied[player];
    let enemy_occupied = bitboards.occupied[other_player(player)];

    let moves = potential & !self_occupied;

    let capture = moves & enemy_occupied;
    let quiet = moves & !capture;

    let start_index = piece_index;

    for end_index in each_index_of_one(capture) {
        let taken_piece = bitboards.piece_at_index(end_index);

        match taken_piece {
            Some(taken_piece) => {
                buffer.push(Move {
                    piece: PlayerPiece { player, piece },
                    start_index,
                    end_index,
                    move_type: MoveType::Capture(Capture::Take { taken_piece }),
                    promotion: None,
                });
            }
            None => {
                return err_result(&format!(
                    "no piece at index {:} but marked as capture",
                    end_index
                ));
            }
        };
    }

    if only_captures == OnlyCaptures::Yes {
        return Ok(());
    }

    for end_index in each_index_of_one(quiet) {
        buffer.push(Move {
            piece: PlayerPiece::new(player, piece),
            start_index,
            end_index,
            move_type: MoveType::Quiet(Quiet::Move),
            promotion: None,
        });
    }

    Ok(())
}

pub fn walk_potential_bb(
    index: BoardIndex,
    all_occupied: Bitboard,
    piece: Piece,
) -> ErrorResult<Bitboard> {
    let walk_types = walk_type_for_piece(piece);

    let walk_types = walk_types?;

    let mut danger_bb = 0;

    for walk_type in walk_types.iter() {
        danger_bb |= moves_bb_for_piece_and_blockers(index, *walk_type, all_occupied);
    }

    Ok(danger_bb)
}

pub fn walk_moves(
    buffer: &mut MoveBuffer,
    PlayerPiece { player, piece }: PlayerPiece,
    bitboards: &Bitboards,
    only_captures: OnlyCaptures,
) -> ErrorResult<()> {
    for piece_index in each_index_of_one(bitboards.pieces[player][piece]) {
        let potential_bb = walk_potential_bb(piece_index, bitboards.all_occupied(), piece);

        match potential_bb {
            Err(err) => return Err(err),
            Ok(potential) => {
                potential_bb_to_moves(
                    buffer,
                    PlayerPiece::new(player, piece),
                    piece_index,
                    potential,
                    bitboards,
                    only_captures,
                )?;
            }
        }
    }

    Ok(())
}

pub fn jumping_bitboard(index: BoardIndex, jumping_piece: JumpingPiece) -> Bitboard {
    let lookup: &[Bitboard; 64] = match jumping_piece {
        JumpingPiece::Knight => &KNIGHT_MOVE_BITBOARD,
        JumpingPiece::King => &KING_MOVE_BITBOARD,
    };
    lookup[index.i]
}

pub fn jump_moves(
    buffer: &mut MoveBuffer,
    player: Player,
    bitboards: &Bitboards,
    jumping_piece: JumpingPiece,
    only_captures: OnlyCaptures,
) -> ErrorResult<()> {
    let piece = match jumping_piece {
        JumpingPiece::Knight => Piece::Knight,
        JumpingPiece::King => Piece::King,
    };
    for piece_index in each_index_of_one(bitboards.pieces[player][piece]) {
        let potential = jumping_bitboard(piece_index, jumping_piece);
        potential_bb_to_moves(
            buffer,
            PlayerPiece::new(player, piece),
            piece_index,
            potential,
            bitboards,
            only_captures,
        )?;
    }

    Ok(())
}

pub fn pawn_attacking_bb(start_bb: Bitboard, capture_dir: Direction) -> Bitboard {
    let start_bb = start_bb & pre_move_mask(capture_dir);
    let attacking_bb = rotate_toward_index_63(start_bb, capture_dir.offset());
    attacking_bb
}

pub fn pawn_capture_move(
    bitboards: &Bitboards,
    player: Player,
    end_index: BoardIndex,
    capture_dir: Direction,
) -> ErrorResult<Move> {
    let start_index = BoardIndex::from((end_index.i as isize - capture_dir.offset()) as usize);
    let taken_piece = bitboards.piece_at_index(end_index);
    match taken_piece {
        Some(taken_piece) => Ok(Move {
            piece: PlayerPiece::new(player, Piece::Pawn),
            start_index,
            end_index,
            move_type: MoveType::Capture(Capture::Take { taken_piece }),
            promotion: None,
        }),
        None => err_result(&format!("no piece at {:} but marked as capture", end_index)),
    }
}

pub fn pawn_quiet_move(
    player: Player,
    end_index: BoardIndex,
    push_dir: Direction,
) -> ErrorResult<Move> {
    let start_index = BoardIndex::from((end_index.i as isize - push_dir.offset()) as usize);
    Ok(Move {
        piece: PlayerPiece::new(player, Piece::Pawn),
        start_index,
        end_index,
        move_type: MoveType::Quiet(Quiet::Move),
        promotion: None,
    })
}

pub fn pawn_moves(
    buffer: &mut MoveBuffer,
    player: Player,
    bitboards: &Bitboards,
    only_captures: OnlyCaptures,
    only_queen_promotion: OnlyQueenPromotion,
) -> ErrorResult<()> {
    let pawns = bitboards.pieces[player][Piece::Pawn];

    let offsets = pawn_capture_directions_for_player(player);
    for capture_dir in offsets {
        let attacking_bb = pawn_attacking_bb(pawns, *capture_dir);
        let capture_bb = attacking_bb & bitboards.occupied[other_player(player)];

        let capture_pawns = capture_bb & !*PAWN_PROMOTION_BITBOARD;
        let promotion_pawns = capture_bb & *PAWN_PROMOTION_BITBOARD;

        for end_index in each_index_of_one(capture_pawns) {
            buffer.push(pawn_capture_move(
                bitboards,
                player,
                end_index,
                *capture_dir,
            )?);
        }
        for end_index in each_index_of_one(promotion_pawns) {
            for &promo_piece in only_queen_promotion.pieces() {
                let mut m = pawn_capture_move(bitboards, player, end_index, *capture_dir)?;
                m.promotion = Some(promo_piece);
                buffer.push(m);
            }
        }
    }

    if let OnlyCaptures::Yes = only_captures {
        return Ok(());
    }

    let quiet_push_dir = pawn_push_direction_for_player(player);

    let masked_pawns = pawns & pre_move_mask(quiet_push_dir);

    let moved_pawns =
        rotate_toward_index_63(masked_pawns, quiet_push_dir.offset()) & !bitboards.all_occupied();

    let pushed_pawns = moved_pawns & !*PAWN_PROMOTION_BITBOARD;
    let promotion_pawns = moved_pawns & *PAWN_PROMOTION_BITBOARD;

    for end_index in each_index_of_one(pushed_pawns) {
        buffer.push(pawn_quiet_move(player, end_index, quiet_push_dir)?);
    }

    for end_index in each_index_of_one(promotion_pawns) {
        for &promo_piece in only_queen_promotion.pieces() {
            let mut m = pawn_quiet_move(player, end_index, quiet_push_dir)?;
            m.promotion = Some(promo_piece);
            buffer.push(m);
        }
    }

    {
        let masked_pawns = pawns & starting_pawns_mask(player);
        let push1 = rotate_toward_index_63(masked_pawns, quiet_push_dir.offset())
            & !bitboards.all_occupied();
        let push2 =
            rotate_toward_index_63(push1, quiet_push_dir.offset()) & !bitboards.all_occupied();

        for end_index in each_index_of_one(push2) {
            let start_index =
                BoardIndex::from((end_index.i as isize - 2 * quiet_push_dir.offset()) as usize);
            let skipped_index =
                BoardIndex::from((end_index.i as isize - quiet_push_dir.offset()) as usize);
            buffer.push(Move {
                piece: PlayerPiece::new(player, Piece::Pawn),
                start_index,
                end_index,
                move_type: MoveType::Quiet(Quiet::PawnSkip { skipped_index }),
                promotion: None,
            })
        }
    }

    Ok(())
}

pub fn en_passant_move(
    buffer: &mut MoveBuffer,
    player: Player,
    bitboards: &Bitboards,
    en_passant_index: Option<BoardIndex>,
) -> ErrorResult<()> {
    let pawns = bitboards.pieces[player][Piece::Pawn];

    if let Some(en_passant_index) = en_passant_index {
        let en_passant_bb = single_bitboard(en_passant_index);

        for (dir, target_dir) in en_passant_move_and_target_offsets(player) {
            let pawns = pawns & pre_move_mask(*dir);
            let moved_pawns = rotate_toward_index_63(pawns, dir.offset());

            if moved_pawns & en_passant_bb != 0 {
                let start_index =
                    BoardIndex::from((en_passant_index.i as isize - dir.offset()) as usize);
                let taken_index =
                    BoardIndex::from((start_index.i as isize + target_dir.offset()) as usize);

                buffer.push(Move {
                    piece: PlayerPiece::new(player, Piece::Pawn),
                    start_index,
                    end_index: en_passant_index,
                    move_type: MoveType::Capture(Capture::EnPassant { taken_index }),
                    promotion: None,
                });
            }
        }
    }

    Ok(())
}

pub fn castling_moves(buffer: &mut MoveBuffer, player: Player, state: &Game) -> ErrorResult<()> {
    let allowed_castling_sides = CASTLING_SIDES
        .iter()
        .filter(move |&&castling_side| state.can_castle[player][castling_side]);

    let castling_requirements = allowed_castling_sides
        .map(move |&castling_side| castling_requirements(player, castling_side));

    let empty_castling_sides = castling_requirements.filter(move |&req| {
        for &empty_index in &req.require_empty {
            if state.board.piece_at_index(empty_index).is_some() {
                return false;
            }
        }
        return true;
    });

    let safe_castling_sides = empty_castling_sides
        .map(move |req| -> ErrorResult<Option<&CastlingRequirements>> {
            for &safe_index in &req.require_safe {
                if index_in_danger(player, safe_index, &state.board)? {
                    return Ok(None);
                }
            }

            Ok(Some(req))
        })
        .filter_map(|req| req.transpose());

    for req in safe_castling_sides {
        let req = req?;
        buffer.push(Move {
            piece: PlayerPiece::new(player, Piece::King),
            start_index: req.king_start,
            end_index: req.king_end,
            move_type: MoveType::Quiet(Quiet::Castle {
                rook_start: req.rook_start,
                rook_end: req.rook_end,
            }),
            promotion: None,
        });
    }

    Ok(())
}

pub fn all_moves<'game>(
    buffer: &mut MoveBuffer,
    player: Player,
    state: &'game Game,
    options: MoveOptions,
) -> ErrorResult<()> {
    buffer.clear();

    pawn_moves(
        buffer,
        player,
        &state.board,
        options.only_captures,
        options.only_queen_promotion,
    )?;
    jump_moves(
        buffer,
        player,
        &state.board,
        JumpingPiece::Knight,
        options.only_captures,
    )?;
    jump_moves(
        buffer,
        player,
        &state.board,
        JumpingPiece::King,
        options.only_captures,
    )?;
    walk_moves(
        buffer,
        PlayerPiece::new(player, Piece::Bishop),
        &state.board,
        options.only_captures,
    )?;
    walk_moves(
        buffer,
        PlayerPiece::new(player, Piece::Rook),
        &state.board,
        options.only_captures,
    )?;
    walk_moves(
        buffer,
        PlayerPiece::new(player, Piece::Queen),
        &state.board,
        options.only_captures,
    )?;
    castling_moves(buffer, player, state)?;

    en_passant_move(buffer, player, &state.board, state.en_passant)?;

    Ok(())
}

pub fn index_in_danger(
    player: Player,
    target: BoardIndex,
    bitboards: &Bitboards,
) -> ErrorResult<bool> {
    let enemy = other_player(player);

    let target_bb = single_bitboard(target);

    let pawn_dangers = pawn_capture_directions_for_player(player).iter().fold(
        0,
        move |danger_bb, capture_dir| -> Bitboard {
            danger_bb | pawn_attacking_bb(target_bb, *capture_dir)
        },
    );

    let knight_dangers = jumping_bitboard(target, JumpingPiece::Knight);
    let king_dangers = jumping_bitboard(target, JumpingPiece::King);

    let bishop_dangers = walk_potential_bb(target, bitboards.all_occupied(), Piece::Bishop);
    let rook_dangers = walk_potential_bb(target, bitboards.all_occupied(), Piece::Rook);

    let bishop_dangers = bishop_dangers?;
    let rook_dangers = rook_dangers?;

    let queen_dangers = bishop_dangers | rook_dangers;

    let enemy_pawns = bitboards.pieces[enemy][Piece::Pawn];
    let enemy_knights = bitboards.pieces[enemy][Piece::Knight];
    let enemy_kings = bitboards.pieces[enemy][Piece::King];
    let enemy_bishops = bitboards.pieces[enemy][Piece::Bishop];
    let enemy_rooks = bitboards.pieces[enemy][Piece::Rook];
    let enemy_queens = bitboards.pieces[enemy][Piece::Queen];

    if enemy_pawns & pawn_dangers != 0
        || enemy_knights & knight_dangers != 0
        || enemy_kings & king_dangers != 0
        || enemy_bishops & bishop_dangers != 0
        || enemy_rooks & rook_dangers != 0
        || enemy_queens & queen_dangers != 0
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_move_to_uci() {
        let m = Move {
            piece: PlayerPiece::new(Player::White, Piece::Pawn),
            start_index: BoardIndex::from(8),
            end_index: BoardIndex::from(16),
            move_type: MoveType::Quiet(Quiet::Move),
            promotion: None,
        };
        assert_eq!(m.to_uci(), "a2a3");
    }

    #[test]
    fn test_castling_repeat_moves() {
        let position = "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 moves e1c1";
        let game = Game::from_position_uci(position).unwrap();

        let mut count_moves = HashMap::<String, usize>::new();

        let mut moves_buffer = MoveBuffer::default();
        all_moves(
            &mut moves_buffer,
            game.player,
            &game,
            MoveOptions::default(),
        )
        .unwrap();

        for m in moves_buffer.iter() {
            let count = count_moves.entry(m.to_uci().to_string()).or_insert(0);
            *count += 1;
        }

        for (m, count) in count_moves.iter() {
            assert_eq!(*count, 1, "incorrect count for: {}", m);
        }
    }

    #[test]
    fn test_promotion_moves() {
        let position =
            "position fen r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1 moves b4c5";
        let game = Game::from_position_uci(position).unwrap();

        let mut count_moves = HashMap::<String, usize>::new();
        let mut moves_buffer = MoveBuffer::default();
        all_moves(
            &mut moves_buffer,
            game.player,
            &game,
            MoveOptions::default(),
        )
        .unwrap();

        for m in moves_buffer.iter() {
            let count = count_moves.entry(m.to_uci().to_string()).or_insert(0);
            *count += 1;
        }

        assert_eq!(*count_moves.get("b2b1b").unwrap(), 1);
        assert_eq!(*count_moves.get("b2b1r").unwrap(), 1);
        assert_eq!(*count_moves.get("b2b1q").unwrap(), 1);
        assert_eq!(*count_moves.get("b2b1n").unwrap(), 1);
    }
}
