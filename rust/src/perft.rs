use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    fs::File,
    io::Write,
    iter,
};

use crate::{
    bitboard::{magic_constants, Bitboard, MAGIC_MOVE_TABLE},
    danger::Danger,
    game::{Game, Legal},
    helpers::{err, err_result, indent, ErrorResult},
    moves::{
        all_moves, index_in_danger, Capture, Move, MoveBuffer, MoveOptions, MoveType, OnlyCaptures,
        OnlyQueenPromotion, Quiet,
    },
    types::{Piece, Player},
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

struct TraverseGameCallbackParams<'game> {
    moves_stack: &'game Vec<Move>,
    game: &'game Game,
    depth: usize,
    max_depth: usize,
}

fn print_game(fen: &str, moves: &Vec<Move>) -> String {
    let mut game = Game::from_fen(fen).unwrap();

    let uci_moves = moves
        .iter()
        .map(|m| m.to_uci())
        .collect::<Vec<_>>()
        .join(" ");

    let mut uci = "".to_string();
    uci.push_str(format!("position fen {} moves {}", fen, uci_moves).as_str());

    let mut s = "".to_string();
    s.push_str(format!("{}\n", game).as_str());

    for m in moves {
        s.push_str(format!("{}\n", m).as_str());
        let result = game.make_move(*m);
        match result {
            Ok(_) => {
                s.push_str(format!("{}\n", game).as_str());
            }
            Err(_) => {
                s.push_str("failed\n");
                break;
            }
        }
    }

    s.push_str(format!("{}", game.to_fen()).as_str());

    format!("'{}': {{\n{}\n}}", uci, indent(&s, 2))
}

fn traverse_game_callback(
    moves_stack: &mut Vec<Move>,
    game: &Game,
    depth: usize,
    max_depth: usize,
    callback: &mut dyn FnMut(&TraverseGameCallbackParams),
) -> ErrorResult<()> {
    callback(&TraverseGameCallbackParams {
        moves_stack,
        game,
        depth,
        max_depth,
    });

    if depth >= max_depth {
        return Ok(());
    }

    for result in game.for_each_legal_move(MoveOptions::default()) {
        let (next_game, m) = result?;
        moves_stack.push(m);
        traverse_game_callback(moves_stack, &next_game, depth + 1, max_depth, callback)?;
        moves_stack.pop();
    }

    Ok(())
}

pub fn run_perft(game: &Game, max_depth: usize) -> ErrorResult<usize> {
    let mut perft_overall = 0;

    let mut moves_stack = vec![];

    traverse_game_callback(&mut moves_stack, &game, 0, max_depth, &mut |params| {
        if params.depth == max_depth {
            perft_overall += 1;

            if params.depth == 0 {
                return;
            }
        }
    })?;

    Ok(perft_overall)
}

pub fn run_perft_counting_first_move(
    game: &Game,
    max_depth: usize,
) -> ErrorResult<(usize, HashMap<String, usize>)> {
    let mut total_count = 0;
    let mut count_per_move: HashMap<String, usize> = HashMap::new();

    for result in game.for_each_legal_move(MoveOptions::default()) {
        let (next_game, next_move) = result?;
        let move_str = next_move.to_uci();
        let count = count_per_move.entry(move_str).or_insert(0);

        *count = run_perft(&next_game, max_depth - 1)?;
        total_count += *count;
    }

    Ok((total_count, count_per_move))
}

fn assert_perft_matches_for_depth(fen: &str, max_depth: usize, expected_count: usize) {
    let game = Game::from_fen(fen).unwrap();
    assert_eq!(game.to_fen(), fen);

    let start_time = std::time::Instant::now();

    let mut perft_overall = 0;

    let mut moves_stack = vec![];

    let result = traverse_game_callback(&mut moves_stack, &game, 0, max_depth, &mut |params| {
        if params.depth == max_depth {
            perft_overall += 1;

            if params.depth == 0 {
                return;
            }
        }
    });

    result.expect(format!("{}", print_game(fen, &moves_stack)).as_str());

    let end_time = std::time::Instant::now();

    println!(
        "calculated perft for max_depth: {}, expected_count: {}, in {} ms",
        max_depth,
        expected_count,
        (end_time - start_time).as_millis()
    );

    assert_eq!(expected_count, perft_overall);
}

fn assert_perft_matches(fen: &str, expected_counts: &[usize]) {
    for (max_depth, &expected_count) in expected_counts.iter().enumerate() {
        assert_perft_matches_for_depth(fen, max_depth, expected_count);
    }
}

#[test]
fn test_perft_start_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    // Run once to warm up the magics cache
    let expected_count = [1, 20];
    assert_perft_matches(fen, &expected_count);

    {
        // let p = Profiler::new("perft_start_board".to_string());
        let expected_count = [
            1, 20, 400, 8902,
            197281,
            // 4865609,
            // 119060324,
            // 3195901860,
        ];
        assert_perft_matches(fen, &expected_count);
        // p.flush();
    }
}

// #[test]
// fn test_perft_start_board_depth_5() {
//     let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
//     let max_depth = 5;
//     let expected_count = 4865609;
//     let expected_branches = HashMap::from([
//         ("a2a3", 181046),
//         ("b2b3", 215255),
//         ("c2c3", 222861),
//         ("d2d3", 328511),
//         ("e2e3", 402988),
//         ("f2f3", 178889),
//         ("g2g3", 217210),
//         ("h2h3", 181044),
//         ("a2a4", 217832),
//         ("b2b4", 216145),
//         ("c2c4", 240082),
//         ("d2d4", 361790),
//         ("e2e4", 405385),
//         ("f2f4", 198473),
//         ("g2g4", 214048),
//         ("h2h4", 218829),
//         ("b1a3", 198572),
//         ("b1c3", 234656),
//         ("g1f3", 233491),
//         ("g1h3", 198502),
//     ]);
//     assert_perft_matches_for_depth(fen, max_depth, expected_count, Some(&expected_branches));
// }

#[test]
fn test_perft_position_2() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let expected_count = [1, 48, 2039, 97862];
    assert_perft_matches(fen, &expected_count);
}

#[test]
fn test_perft_position_3() {
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let expected_count = [1, 14, 191, 2812];
    assert_perft_matches(fen, &expected_count);
}

#[test]
fn test_perft_position_4() {
    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    let expected_count = [1, 6, 264, 9467];
    assert_perft_matches(fen, &expected_count);
}

#[test]
fn test_perft_position_5() {
    let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let expected_count = [1, 44, 1486, 62379];
    assert_perft_matches(fen, &expected_count);
}

#[test]
fn test_perft_position_6() {
    let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
    let expected_count = [1, 46, 2079, 89890];
    assert_perft_matches(fen, &expected_count);
}

#[derive(Default, Copy, Clone)]
struct IndexedMoveBuffer {
    buffer: MoveBuffer,
    index: usize,
}

impl Debug for IndexedMoveBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut lines = self.buffer.moves[..self.buffer.size()]
            .iter()
            .map(|m| format!("{:?}", m))
            .collect::<Vec<_>>();

        if self.index < self.buffer.size() {
            lines[self.index] = format!("{} <=========", lines[self.index]);
        }

        f.debug_struct("IndexedMoveBuffer")
            .field("moves", &lines)
            .finish()
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct PerftStackFrame {
    game: Game,
    danger: Danger,
    last_move: Option<Move>,

    moves: Option<IndexedMoveBuffer>,
}

impl PerftStackFrame {
    pub fn lazily_generate_moves(&mut self) -> ErrorResult<()> {
        if self.moves.is_some() {
            return Ok(());
        }

        self.moves = Some(IndexedMoveBuffer {
            buffer: MoveBuffer::default(),
            index: 0,
        });

        let moves = self
            .moves
            .as_mut()
            .ok_or_else(|| err("moves should be generated"))?;

        self.game.fill_pseudo_move_buffer(
            &mut moves.buffer,
            MoveOptions {
                only_captures: OnlyCaptures::No,
                only_queen_promotion: OnlyQueenPromotion::No,
            },
        )?;
        moves.index = 0;

        Ok(())
    }

    pub fn setup_from_previous(
        &mut self,
        previous: &PerftStackFrame,
        m: &Move,
    ) -> ErrorResult<Legal> {
        self.game = previous.game;
        self.game.make_move(*m)?;

        if self.game.move_legality(m, &previous.danger) == Legal::No {
            return Ok(Legal::No);
        }

        self.danger = Danger::from(self.game.player, &self.game.board)?;
        self.last_move = Some(*m);

        Ok(Legal::Yes)
    }

    pub fn setup_from_scratch(&mut self, game: Game) -> ErrorResult<()> {
        self.game = game;
        self.danger = Danger::from(self.game.player, &self.game.board).unwrap();
        self.last_move = None;

        self.lazily_generate_moves()?;

        Ok(())
    }
}

struct PerftData {
    stack: [PerftStackFrame; 40],
    depth: usize,
}

impl Debug for PerftData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerftData")
            .field("depth", &self.depth)
            .field("previous", &self.previous())
            .field("current", &self.current().unwrap())
            .finish()
    }
}

impl PerftData {
    fn new(game: Game) -> ErrorResult<Self> {
        let mut data = Self {
            stack: [PerftStackFrame::default(); 40],
            depth: 0,
        };
        let start = &mut data.stack[0];
        start.setup_from_scratch(game)?;

        Ok(data)
    }

    fn current(&self) -> ErrorResult<&PerftStackFrame> {
        self.stack
            .get(self.depth)
            .ok_or(err("current index invalid"))
    }

    fn current_mut(&mut self) -> ErrorResult<&mut PerftStackFrame> {
        self.stack
            .get_mut(self.depth)
            .ok_or(err("current index invalid"))
    }

    fn previous(&self) -> Option<&PerftStackFrame> {
        if self.depth == 0 {
            return None;
        }
        self.stack.get(self.depth - 1)
    }

    fn current_and_next_mut(
        &mut self,
    ) -> ErrorResult<(&mut PerftStackFrame, &mut PerftStackFrame)> {
        let (current, next) = self.stack.split_at_mut(self.depth + 1);
        let current = current.last_mut().ok_or(err("current index invalid"))?;
        let next = next.first_mut().ok_or(err("next index invalid"))?;
        Ok((current, next))
    }

    fn previous_and_current_mut(
        &mut self,
    ) -> ErrorResult<(Option<&mut PerftStackFrame>, &mut PerftStackFrame)> {
        let (previous, current) = self.stack.split_at_mut(self.depth);
        let previous = previous.last_mut();
        let current = current.first_mut().ok_or(err("current index invalid"))?;
        Ok((previous, current))
    }

    fn next_move(&mut self) -> ErrorResult<Option<Move>> {
        let current = self.current_mut()?;
        current.lazily_generate_moves()?;

        let current_moves = current
            .moves
            .as_mut()
            .ok_or_else(|| err("moves should be generated"))?;

        if current_moves.index >= current_moves.buffer.size() {
            return Ok(None);
        }

        let m = current_moves.buffer.get(current_moves.index);
        current_moves.index += 1;

        Ok(Some(*m))
    }

    fn make_move(&mut self, m: &Move) -> ErrorResult<()> {
        let (current, next) = self.current_and_next_mut()?;

        if next.setup_from_previous(current, m)? == Legal::No {
            return Ok(());
        }

        return Ok(());
    }
}

pub fn run_perft_iteratively(
    game: Game,
    max_depth: usize,
    num_iterations: usize,
) -> ErrorResult<usize> {
    let mut data = PerftData::new(game)?;

    let mut overall_count = 0;

    if max_depth <= 1 {
        return Ok(1);
    }

    for _ in 0..num_iterations {
        // println!("{:#?}", data);
        let current_depth = data.depth;

        // Leaf node case:
        if current_depth + 1 >= max_depth {
            println!("LEAF {:#?}", data.current()?.game);
            overall_count += 1;
            data.depth -= 1;
            continue;
        }

        // We have moves to traverse, dig deeper
        let next_move = data.next_move()?;
        if let Some(next_move) = next_move {
            println!(
                "DIG performing {:#?} at depth {}",
                next_move,
                current_depth + 1
            );
            data.make_move(&next_move)?;
            data.depth += 1;

            println!("{:#?}", data.current()?.game);
            continue;
        }

        // We're out of moves to traverse, pop back up.
        if current_depth == 0 {
            println!("DONE {:#?}", data.current()?.game);
            break;
        } else {
            println!("NO MOVES {:#?}", data.current()?.game);
            data.depth -= 1;
            continue;
        }
    }

    Ok(overall_count)
}

#[test]
fn test_perft_start_board_iteratively() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let expected_count = [
        1, 20, 400, 8902, 197281,
        // 4865609,
        // 119060324,
        // 3195901860,
    ];

    for (i, expected_count) in
        expected_count.into_iter().enumerate().collect::<Vec<_>>()[2..].into_iter()
    {
        let max_depth = i + 1;
        let max_iterations = max_depth * expected_count;

        let count =
            run_perft_iteratively(Game::from_fen(fen).unwrap(), max_depth, max_iterations).unwrap();
        assert_eq!(count, *expected_count);
    }
}
