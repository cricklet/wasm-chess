use std::fmt::Debug;
use std::fmt::Formatter;

use super::danger::Danger;
use super::game::Game;
use super::game::Legal;
use super::helpers::err_result;
use super::helpers::ErrorResult;
use super::helpers::OptionResult;
use super::moves::Move;
use super::moves::MoveBuffer;
use super::moves::MoveOptions;
use super::moves::OnlyCaptures;
use super::moves::OnlyQueenPromotion;

#[derive(PartialEq, Eq)]
pub enum FinishedTraversing {
    No,
    Yes,
}

#[derive(Default, Copy, Clone)]
pub struct IndexedMoveBuffer {
    buffer: MoveBuffer,
    index: usize,
}

impl Debug for IndexedMoveBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut lines = self.buffer.moves[..self.buffer.size]
            .iter()
            .map(|m| format!("{:?}", m))
            .collect::<Vec<_>>();

        if self.index < self.buffer.size {
            lines[self.index] = format!("{} <=========", lines[self.index]);
        }

        f.debug_struct("IndexedMoveBuffer")
            .field("moves", &lines)
            .finish()
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct TraversalStackFrame {
    game: Game,
    last_move: Option<Move>,

    danger: Option<Danger>,
    moves: Option<IndexedMoveBuffer>,
}

impl TraversalStackFrame {
    pub fn setup_from_move(
        &mut self,
        previous: &mut TraversalStackFrame,
        next_move: &Move,
    ) -> ErrorResult<Legal> {
        self.game = previous.game;
        self.game.make_move(*next_move)?;

        self.danger = None;
        self.moves = None;

        previous.lazily_generate_danger()?;

        if self
            .game
            .move_legality(next_move, previous.danger.as_ref().as_result()?)
            == Legal::No
        {
            return Ok(Legal::No);
        }

        self.last_move = Some(*next_move);

        Ok(Legal::Yes)
    }

    pub fn setup_from_scratch(&mut self, game: Game) -> ErrorResult<()> {
        self.game = game;
        self.last_move = None;

        self.danger = None;
        self.moves = None;

        self.lazily_generate_danger()?;
        self.lazily_generate_moves()?;

        Ok(())
    }
    pub fn lazily_generate_moves(&mut self) -> ErrorResult<&IndexedMoveBuffer> {
        if self.moves.is_some() {
            return self.moves.as_ref().as_result();
        }

        self.moves = Some(IndexedMoveBuffer {
            buffer: MoveBuffer::default(),
            index: 0,
        });

        let moves = self.moves.as_mut().as_result()?;

        self.game.fill_pseudo_move_buffer(
            &mut moves.buffer,
            MoveOptions {
                only_captures: OnlyCaptures::No,
                only_queen_promotion: OnlyQueenPromotion::No,
            },
        )?;
        moves.index = 0;

        Ok(self.moves.as_ref().unwrap())
    }

    pub fn lazily_generate_danger(&mut self) -> ErrorResult<&Danger> {
        if self.danger.is_some() {
            return Ok(self.danger.as_ref().unwrap());
        }

        self.danger = Some(Danger::from(self.game.player, &self.game.board)?);
        Ok(self.danger.as_ref().unwrap())
    }
}

pub struct TraversalStack<const N: usize> {
    stack: [TraversalStackFrame; N],
    pub depth: usize,
}

impl<const N: usize> Debug for TraversalStack<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraversalStack")
            .field("depth", &self.depth)
            .field("previous", &self.previous())
            .field("current", &self.current().unwrap())
            .finish()
    }
}

impl<const N: usize> TraversalStack<N> {
    pub fn new(game: Game) -> ErrorResult<Self> {
        let mut data = Self {
            stack: [TraversalStackFrame::default(); N],
            depth: 0,
        };
        let start = &mut data.stack[0];
        start.setup_from_scratch(game)?;

        Ok(data)
    }

    pub fn current(&self) -> ErrorResult<&TraversalStackFrame> {
        self.stack.get(self.depth).as_result()
    }

    fn current_mut(&mut self) -> ErrorResult<&mut TraversalStackFrame> {
        self.stack.get_mut(self.depth).as_result()
    }

    fn previous(&self) -> Option<&TraversalStackFrame> {
        if self.depth == 0 {
            return None;
        }
        self.stack.get(self.depth - 1)
    }

    pub fn current_and_next_mut(
        &mut self,
    ) -> ErrorResult<(&mut TraversalStackFrame, &mut TraversalStackFrame)> {
        if let Some((current, remainder)) = self.stack[self.depth..].split_first_mut() {
            Ok((current, remainder.first_mut().unwrap()))
        } else {
            err_result("current index invalid")
        }
    }

    pub fn next_move(&mut self) -> ErrorResult<Option<Move>> {
        let current = self.current_mut()?;
        current.lazily_generate_moves()?;

        let current_moves = current.moves.as_mut().as_result()?;

        if current_moves.index >= current_moves.buffer.size {
            return Ok(None);
        }

        let m = current_moves.buffer.get(current_moves.index);
        current_moves.index += 1;

        Ok(Some(*m))
    }
}
