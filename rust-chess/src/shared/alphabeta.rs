// use crate::iterative_traversal::TraversalStack;

// use super::{
//     danger::Danger,
//     evaluation::*,
//     game::{Game, Legal},
//     helpers::ErrorResult,
//     moves::*,
//     types::*,
// };

// struct MoveHistory<'h> {
//     history: &'h mut Vec<Move>,
// }

// impl<'h> MoveHistory<'h> {
//     pub fn track(history: &'h mut Vec<Move>, m: Move) -> Self {
//         history.push(m);
//         MoveHistory { history }
//     }
// }

// impl<'h> Drop for MoveHistory<'h> {
//     fn drop(&mut self) {
//         self.history.pop();
//     }
// }
// // ************************************************************************************************* //

// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub enum Evaluation {
//     Unknown,
//     Centipawns(Player, isize),
//     WinInN(Player, usize),
// }

// impl Default for Evaluation {
//     fn default() -> Self { Evaluation::Unknown }
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub enum Comparison {
//     Better,
//     Equal,
//     Worse,
//     Unknown,
// }

// impl Comparison {
//     pub fn is_better_or_equal(self) -> bool {
//         self == Comparison::Better || self == Comparison::Equal
//     }
//     pub fn is_better(self) -> bool {
//         self == Comparison::Better
//     }
// }

// impl Evaluation {
//     fn comparison_points(&self, current_player: Player) -> Option<(isize, isize)> {
//         match self {
//             Evaluation::Centipawns(player, score) => {
//                 if *player == current_player {
//                     Some((0, *score))
//                 } else {
//                     Some((0, -*score))
//                 }
//             }
//             Evaluation::WinInN(player, n) => {
//                 if *player == current_player {
//                     Some((1000 - *n as isize, 0))
//                 } else {
//                     Some((-1000 + *n as isize, 0))
//                 }
//             }
//             Evaluation::Unknown => None,
//         }
//     }

//     pub fn compare(current_player: Player, left: Evaluation, right: Evaluation) -> Comparison {
//         let left_points = left.comparison_points(current_player);
//         let right_points = right.comparison_points(current_player);

//         if left_points.is_none() || right_points.is_none() {
//             return Comparison::Unknown;
//         }
//         let (left_mate, left_eval) = left_points.unwrap();
//         let (right_mate, right_eval) = right_points.unwrap();

//         if left_mate > right_mate {
//             Comparison::Better
//         } else if left_mate < right_mate {
//             Comparison::Worse
//         } else if left_eval > right_eval {
//             Comparison::Better
//         } else if left_eval < right_eval {
//             Comparison::Worse
//         } else {
//             Comparison::Equal
//         }
//     }
// }

// #[test]
// fn test_evaluation_comparison() {
//     assert_eq!(
//         Evaluation::compare(
//             Player::White,
//             Evaluation::Centipawns(Player::White, 0),
//             Evaluation::Centipawns(Player::White, 0)
//         ),
//         Comparison::Equal
//     );
//     assert_eq!(
//         Evaluation::compare(
//             Player::White,
//             Evaluation::Centipawns(Player::White, 100),
//             Evaluation::Centipawns(Player::White, 0)
//         ),
//         Comparison::Better
//     );
//     assert_eq!(
//         Evaluation::compare(
//             Player::White,
//             Evaluation::Centipawns(Player::White, 100),
//             Evaluation::Centipawns(Player::White, 200)
//         ),
//         Comparison::Worse
//     );
//     assert_eq!(
//         Evaluation::compare(
//             Player::White,
//             Evaluation::Centipawns(Player::Black, 100),
//             Evaluation::Centipawns(Player::White, 0)
//         ),
//         Comparison::Worse
//     );
//     assert_eq!(
//         Evaluation::compare(
//             Player::White,
//             Evaluation::Centipawns(Player::Black, -300),
//             Evaluation::Centipawns(Player::White, 200)
//         ),
//         Comparison::Better
//     );
//     assert_eq!(
//         Evaluation::compare(
//             Player::White,
//             Evaluation::WinInN(Player::White, 0),
//             Evaluation::WinInN(Player::White, 1),
//         ),
//         Comparison::Better
//     );
//     assert_eq!(
//         Evaluation::compare(
//             Player::Black,
//             Evaluation::WinInN(Player::White, 0),
//             Evaluation::WinInN(Player::White, 1),
//         ),
//         Comparison::Worse
//     );
//     assert_eq!(
//         Evaluation::compare(
//             Player::White,
//             Evaluation::WinInN(Player::White, 1),
//             Evaluation::WinInN(Player::Black, 1),
//         ),
//         Comparison::Better
//     );
//     assert_eq!(
//         Evaluation::compare(
//             Player::Black,
//             Evaluation::WinInN(Player::White, 1),
//             Evaluation::WinInN(Player::Black, 1),
//         ),
//         Comparison::Worse
//     );
// }

// // ************************************************************************************************* //

// const MAX_ALPHA_BETA_DEPTH: usize = 40;

// #[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
// struct AlphaBetaStackData {
//     best_move: Option<(Move, Evaluation)>,

//     alpha: Evaluation,
//     beta: Evaluation,

//     in_quiescence: InQuiescence,
// }


// #[derive(Debug, PartialEq, Eq)]
// pub enum LoopResult {
//     Continue,
//     Done,
// }
// struct AlphaBeta {
//     pub traversal: TraversalStack<AlphaBetaStackData, MAX_ALPHA_BETA_DEPTH>,
//     pub max_depth: usize,
// }

// impl AlphaBeta {
//     pub fn new(game: Game) -> ErrorResult<Self> {
//         Ok(Self {
//             traversal: TraversalStack::<AlphaBetaStackData, MAX_ALPHA_BETA_DEPTH>::new(game)?,
//             max_depth: 3,
//         })
//     }

//     pub fn iterate(&mut self) -> ErrorResult<LoopResult> {
//         let next_move = self.traversal.get_and_increment_move().unwrap();

//         // We have moves to traverse, dig deeper
//         if let Some(next_move) = next_move {
//             let (current_depth, next_depth) = (self.traversal.depth, self.traversal.depth + 1);
//             let (current, next) = self.traversal.current_and_next_mut().unwrap();

//             let result = next.setup_from_move(current, &next_move).unwrap();
//             if result == Legal::No {
//                 return Ok(LoopResult::Continue);
//             }

//             // We're about to search a leaf node
//             if next_depth >= self.max_depth {
//                 let next_game = next.game;
//                 let score = Evaluation::Centipawns(next_game.player, evaluate(&next_game, next_game.player));

//                 if Evaluation::compare(player, score, beta).is_better_or_equal() {
//                     // enemy is can force a better score. cutoff early.
//                     // beta is the lower bound for the score we can get at this board state.
//                     return Ok(beta);
//                 }

//                 if best_score.is_none()
//                     || Evaluation::compare(player, score, best_score.unwrap()).is_better()
//                 {
//                     best_score = Some(score);
//                     if Evaluation::compare(player, best_score.unwrap(), alpha).is_better() {
//                         // enemy won't prevent us from making this move. keep searching.
//                         alpha = best_score.unwrap();
//                     }
//                 }
//             }

//             self.traversal.depth += 1;
//             return Ok(LoopResult::Continue);
//         }

//         // We're out of moves to traverse, pop back up.
//         if self.traversal.depth == 0 {
//             return Ok(LoopResult::Done);
//         } else {
//             self.traversal.depth -= 1;
//             return Ok(LoopResult::Continue);
//         }
//     }

//     fn is_quiet_position(&self, danger: &Danger, last_move: Option<&Move>) -> bool {
//         if danger.check {
//             return false;
//         }

//         if let Some(last_move) = last_move {
//             if !last_move.is_quiet() {
//                 return false;
//             }
//         }

//         true
//     }
// }

// // ************************************************************************************************* //

// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// enum InQuiescence {
//     No,
//     Yes,
// }

// impl Default for InQuiescence {
//     fn default() -> Self { InQuiescence::No }
// }


// impl InQuiescence {
//     fn move_options(self) -> MoveOptions {
//         match self {
//             InQuiescence::No => MoveOptions {
//                 only_captures: OnlyCaptures::No,
//                 only_queen_promotion: OnlyQueenPromotion::No,
//             },
//             InQuiescence::Yes => MoveOptions {
//                 only_captures: OnlyCaptures::Yes,
//                 only_queen_promotion: OnlyQueenPromotion::No,
//             },
//         }
//     }
// }
