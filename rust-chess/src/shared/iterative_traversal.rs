use std::fmt::Debug;
use std::fmt::Formatter;

use crate::danger::LazyDanger;
use crate::helpers::indent;
use crate::helpers::StableOption;
use crate::moves::LazyMoves;

use super::danger::Danger;
use super::game::Game;
use super::game::Legal;
use super::helpers::err_result;
use super::helpers::ErrorResult;
use super::helpers::OptionResult;
use super::moves::Move;
use super::moves::MoveOptions;
use super::moves::OnlyCaptures;
use super::moves::OnlyQueenPromotion;

#[derive(Default, Debug)]
pub struct TraversalStackFrame<D> {
    pub game: Game,

    danger: LazyDanger,
    pub moves: LazyMoves,

    pub history_move: Option<Move>,

    pub data: D,
}

impl<D: Debug> TraversalStackFrame<D> {
    pub fn danger(&mut self) -> ErrorResult<&Danger> {
        self.danger.get(self.game.player(), self.game.bitboards())
    }

    pub fn recent_move(&self) -> ErrorResult<Option<Move>> {
        self.moves.last()
    }

    pub fn setup(
        &mut self,
        previous: &mut TraversalStackFrame<D>,
        move_to_apply: &Move,
    ) -> ErrorResult<Legal> {
        self.game = previous.game;
        self.game.make_move(*move_to_apply)?;

        self.danger.reset();
        self.moves.reset();

        self.history_move = Some(move_to_apply.clone());

        if self.game.move_legality(move_to_apply, previous.danger()?) == Legal::No {
            return Ok(Legal::No);
        }

        Ok(Legal::Yes)
    }

    pub fn setup_from_scratch(&mut self, game: Game) -> ErrorResult<()> {
        self.game = game;

        self.danger.reset();
        self.moves.reset();

        Ok(())
    }
}

pub struct TraversalStack<D: Debug, const N: usize> {
    stack: [TraversalStackFrame<D>; N],
    pub depth: usize,
}

impl<D: Debug, const N: usize> Debug for TraversalStack<D, N> {
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

impl<D: Debug, const N: usize> TraversalStack<D, N> {
    pub fn new<F: Fn(usize) -> D>(game: Game, data_callback: F) -> ErrorResult<Self> {
        let mut data = Self {
            stack: std::array::from_fn::<_, N, _>(|i| TraversalStackFrame::<D> {
                game: Game::default(),
                danger: LazyDanger::default(),
                moves: LazyMoves::default(),
                history_move: None,
                data: data_callback(i),
            }),
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
            Ok((current, remainder.first_mut().as_result()?))
        } else {
            err_result("current index invalid")
        }
    }

}

pub fn null_move_sort(_game: &Game, _moves: &mut Vec<Move>) -> ErrorResult<()> {
    Ok(())
}
