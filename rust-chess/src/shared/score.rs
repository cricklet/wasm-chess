use crate::types::Player;
use std::fmt::Display;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Score {
    Centipawns(Player, isize),
    WinInN(Player, usize),
    DrawInN(usize),
}

impl Default for Score {
    fn default() -> Self {
        Score::Centipawns(Player::White, 0)
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Score::Centipawns(player, score) => match player {
                Player::White => write!(f, "{}", score),
                Player::Black => write!(f, "{}", -score),
            },
            Score::WinInN(player, n) => write!(f, "{} wins +{} mate", player.to_fen(), n),
            Score::DrawInN(n) => write!(f, "draw +{}", n),
        }
    }
}

impl std::fmt::Debug for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Score {
    pub fn aspiration_window(self, for_player: Player) -> (Self, Self) {
        match self {
            Score::Centipawns(player, score) => {
                let offset = if player == for_player { 110 } else { -110 };
                (
                    Score::Centipawns(player, score - offset),
                    Score::Centipawns(player, score + offset),
                )
            }
            Score::WinInN(_, _) => (
                Score::WinInN(for_player.other(), 0),
                Score::WinInN(for_player, 0),
            ),
            Score::DrawInN(_) => (
                Score::WinInN(for_player.other(), 0),
                Score::WinInN(for_player, 0),
            ),
        }
    }

    pub fn increment_turns(self) -> Self {
        let mut new_score = self;
        match new_score {
            Score::WinInN(_, ref mut i) => *i += 1,
            Score::DrawInN(ref mut i) => *i += 1,
            Score::Centipawns(..) => {}
        }
        new_score
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Comparison {
    Better,
    Equal,
    Worse,
    Unknown,
}

impl Comparison {
    pub fn is_better_or_equal(self) -> bool {
        self == Comparison::Better || self == Comparison::Equal
    }
    pub fn is_better(self) -> bool {
        self == Comparison::Better
    }
    pub fn is_worse(self) -> bool {
        self == Comparison::Worse
    }
}

impl Score {
    fn comparison_points(&self, current_player: Player) -> Option<(isize, isize)> {
        match self {
            Score::Centipawns(player, score) => {
                if *player == current_player {
                    Some((0, *score))
                } else {
                    Some((0, -*score))
                }
            }
            Score::WinInN(player, n) => {
                if *player == current_player {
                    Some((999999 - *n as isize, 0))
                } else {
                    Some((-99999 + *n as isize, 0))
                }
            }
            Score::DrawInN(_) => Some((0, -50)),
        }
    }

    fn is_draw(&self) -> bool {
        match self {
            Score::DrawInN(_) => true,
            _ => false,
        }
    }

    pub fn compare(current_player: Player, left: Score, right: Score) -> Comparison {
        let left_points = left.comparison_points(current_player);
        let right_points = right.comparison_points(current_player);

        if left_points.is_none() || right_points.is_none() {
            return Comparison::Unknown;
        }
        let (left_mate, left_eval) = left_points.unwrap();
        let (right_mate, right_eval) = right_points.unwrap();

        if left_mate > right_mate {
            Comparison::Better
        } else if left_mate < right_mate {
            Comparison::Worse
        } else if left_eval > right_eval {
            Comparison::Better
        } else if left_eval < right_eval {
            Comparison::Worse
        } else {
            Comparison::Equal
        }
    }
}

#[test]
fn test_evaluation_comparison() {
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::White, 0),
            Score::Centipawns(Player::White, 0)
        ),
        Comparison::Equal
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::White, 100),
            Score::Centipawns(Player::White, 0)
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::White, 100),
            Score::Centipawns(Player::White, 200)
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::Black, 100),
            Score::Centipawns(Player::White, 0)
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::Black, -300),
            Score::Centipawns(Player::White, 200)
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 0),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::WinInN(Player::White, 0),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 1),
            Score::WinInN(Player::Black, 1),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::WinInN(Player::White, 1),
            Score::WinInN(Player::Black, 1),
        ),
        Comparison::Worse
    );

    // Don't prefer a draw if you're only losing by a little
    assert_eq!(
        Score::compare(
            Player::White,
            Score::DrawInN(0),
            Score::Centipawns(Player::Black, 20),
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::DrawInN(0),
            Score::Centipawns(Player::Black, 500),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::DrawInN(0),
            Score::Centipawns(Player::White, -500),
        ),
        Comparison::Better
    );

    // Don't let them draw if you're winning
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::Centipawns(Player::Black, 100),
            Score::DrawInN(0),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::Centipawns(Player::Black, 500),
            Score::DrawInN(0),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::DrawInN(0),
            Score::Centipawns(Player::Black, 500),
        ),
        Comparison::Worse
    );

    // Mates beat draws
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::DrawInN(0),
            Score::WinInN(Player::Black, 1),
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::DrawInN(0),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Better
    );
}

#[test]
fn test_evaluation_increment() {
    // It is better to win sooner
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 0),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 0).increment_turns(),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Equal
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 0)
                .increment_turns()
                .increment_turns(),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Worse
    );
}
