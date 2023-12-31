use std::fmt::Debug;
use std::fmt::Formatter;

use crate::danger::LazyDanger;
use crate::helpers::indent;
use crate::helpers::Error;
use crate::helpers::StableOption;
use crate::moves::LazyMoves;
use crate::zobrist::IsDraw;
use crate::zobrist::ZobristHistory;

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

pub trait TraversalData: Debug + Default {
    fn setup(&mut self, previous: &Self);
}

#[derive(Default, Debug)]
pub struct TraversalStackFrame<D: Debug + TraversalData> {
    pub game: Game,

    danger: LazyDanger,
    pub moves: LazyMoves,

    pub history_move: Option<Move>,

    pub data: D,
}

impl<D: Debug + TraversalData> TraversalStackFrame<D> {
    pub fn danger(&mut self) -> ErrorResult<Danger> {
        self.danger
            .get(self.game.player(), self.game.bitboards())
            .cloned()
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

        self.data.setup(&previous.data);

        if self.game.move_legality(move_to_apply, &previous.danger()?) == Legal::No {
            return Ok(Legal::No);
        }

        Ok(Legal::Yes)
    }
}

pub struct TraversalStack<D: TraversalData> {
    stack: Vec<TraversalStackFrame<D>>,
    depth: usize,
    zobrist_history: ZobristHistory,
}

impl<D: TraversalData> Debug for TraversalStack<D> {
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

impl<D: TraversalData> TraversalStack<D> {
    pub fn new(game: Game, data: D, starting_history: ZobristHistory) -> ErrorResult<Self> {
        let data = Self {
            stack: vec![
                TraversalStackFrame::<D> {
                    game,
                    danger: LazyDanger::default(),
                    moves: LazyMoves::default(),
                    history_move: None,
                    data,
                },
                Default::default(),
            ],
            depth: 0,
            zobrist_history: starting_history,
        };

        Ok(data)
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn is_draw_by_repetition(&self) -> bool {
        self.zobrist_history.is_draw() == IsDraw::Yes
    }

    pub fn increment_depth(&mut self) -> ErrorResult<()> {
        self.depth += 1;

        let (current, _) = self.current()?;
        self.zobrist_history.add(current.game.zobrist());

        if self.depth + 1 >= self.stack.len() {
            self.stack.push(Default::default());
        }

        Ok(())
    }

    pub fn decrement_depth(&mut self) -> ErrorResult<()> {
        self.zobrist_history.pop()?;
        self.depth -= 1;
        Ok(())
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

    pub fn history_display_string(&self) -> ErrorResult<String> {
        let mut result = "".to_string();
        for i in 1..=self.depth {
            let frame = self.stack.get(i).as_result()?;
            let history_move = frame.history_move.as_ref().as_result()?;
            result += &format!("{} ", history_move);
        }
        Ok(result.trim().to_string())
    }

    pub fn history_uci_string(&self) -> ErrorResult<String> {
        let mut result = "".to_string();
        for i in 1..=self.depth {
            let frame = self.stack.get(i).as_result()?;
            let history_move = frame.history_move.as_ref().as_result()?;
            result += &format!("{} ", history_move.to_uci());
        }
        Ok(result.trim().to_string())
    }
}

pub fn null_move_sort(_game: &Game, _moves: &mut [Move]) -> ErrorResult<()> {
    Ok(())
}
