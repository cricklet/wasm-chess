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
pub struct TraversalStackFrame<D> {
    pub game: Game,

    pub danger: Option<Danger>,
    pub moves: Option<IndexedMoveBuffer>,

    pub data: D,
}

impl<D> TraversalStackFrame<D> {
    pub fn setup_from_move(
        &mut self,
        previous: &mut TraversalStackFrame<D>,
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

        Ok(Legal::Yes)
    }

    pub fn setup_from_scratch(&mut self, game: Game) -> ErrorResult<()> {
        self.game = game;

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

    pub fn get_and_increment_move(&mut self) -> ErrorResult<Option<Move>> {
        self.lazily_generate_moves()?;

        let current_moves = self.moves.as_mut().as_result()?;

        if current_moves.index >= current_moves.buffer.size {
            return Ok(None);
        }

        let m = current_moves.buffer.get(current_moves.index);
        current_moves.index += 1;

        Ok(Some(*m))
    }
}

pub struct TraversalStack<D: Debug + Default + Copy, const N: usize> {
    stack: [TraversalStackFrame<D>; N],
    pub depth: usize,
}

impl<D: Debug + Default + Copy, const N: usize> Debug for TraversalStack<D, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("TraversalStack");
        debug.field("depth", &self.depth);

        let current_depth = self.depth;
        if current_depth >= 1 {
            let previous_depth = self.depth - 1;
            debug.field("previous", &self.stack.get(previous_depth).unwrap());
        }
        debug.field("current", &self.stack.get(current_depth).unwrap());
        debug.finish()
    }
}

impl<D: Debug + Default + Copy, const N: usize> TraversalStack<D, N> {
    pub fn new(game: Game) -> ErrorResult<Self> {
        let mut data = Self {
            stack: [TraversalStackFrame::default(); N],
            depth: 0,
        };
        let start = &mut data.stack[0];
        start.setup_from_scratch(game)?;

        Ok(data)
    }

    pub fn root(&self) -> &TraversalStackFrame<D> {
        self.stack.get(0).unwrap()
    }

    pub fn current(&self) -> ErrorResult<(&TraversalStackFrame<D>, usize)> {
        let current_depth = self.depth;
        Ok((self.stack.get(current_depth).as_result()?, current_depth))
    }

    pub fn current_mut(&mut self) -> ErrorResult<(&mut TraversalStackFrame<D>, usize)> {
        let current_depth = self.depth;
        Ok((
            self.stack.get_mut(current_depth).as_result()?,
            current_depth,
        ))
    }

    pub fn current_depth(&self) -> usize {
        self.depth
    }

    pub fn next(&self) -> ErrorResult<(&TraversalStackFrame<D>, usize)> {
        let next_depth = self.depth + 1;
        Ok((self.stack.get(next_depth).as_result()?, next_depth))
    }

    pub fn next_mut(&mut self) -> ErrorResult<(&mut TraversalStackFrame<D>, usize)> {
        let next_depth = self.depth + 1;
        Ok((self.stack.get_mut(next_depth).as_result()?, next_depth))
    }

    fn previous(&self) -> ErrorResult<Option<(&TraversalStackFrame<D>, usize)>> {
        if self.depth == 0 {
            return Ok(None);
        }
        let previous_depth = self.depth - 1;
        Ok(Some((
            self.stack.get(previous_depth).as_result()?,
            previous_depth,
        )))
    }

    pub fn current_and_next_mut(
        &mut self,
    ) -> ErrorResult<(&mut TraversalStackFrame<D>, &mut TraversalStackFrame<D>)> {
        if let Some((current, remainder)) = self.stack[self.depth..].split_first_mut() {
            Ok((current, remainder.first_mut().unwrap()))
        } else {
            err_result("current index invalid")
        }
    }

    pub fn get_and_increment_move(&mut self) -> ErrorResult<Option<Move>> {
        let (current, _) = self.current_mut()?;
        current.get_and_increment_move()
    }

    pub fn move_applied_before_depth(&self, depth: usize) -> ErrorResult<Option<Move>> {
        if depth == 0 {
            // eg if we're searching from startpos, there's no previous move
            // to get to that state.
            return Ok(None);
        }

        let node = self.stack.get(depth - 1).as_result()?;
        match node.moves {
            None => err_result(&format!("no moves at previous depth {} to get to {}, {:#?}", depth - 1, depth, self))?,
            Some(moves) => {
                if moves.index == 0 {
                    return Ok(None);
                }
                Ok(Some(*moves.buffer.get(moves.index - 1)))
            }
        }
    }
}
