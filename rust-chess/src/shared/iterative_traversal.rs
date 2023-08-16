use std::fmt::Debug;
use std::fmt::Formatter;

use crate::helpers::StableOption;
use crate::helpers::indent;

use super::danger::Danger;
use super::game::Game;
use super::game::Legal;
use super::helpers::err_result;
use super::helpers::Clearable;
use super::helpers::ErrorResult;
use super::helpers::OptionResult;
use super::moves::Move;
use super::moves::MoveOptions;
use super::moves::OnlyCaptures;
use super::moves::OnlyQueenPromotion;

#[derive(PartialEq, Eq)]
pub enum FinishedTraversing {
    No,
    Yes,
}

#[derive(Default)]
pub struct IndexedMoveBuffer {
    buffer: Vec<Move>,
    index: usize,
}

impl Clearable for IndexedMoveBuffer {
    fn clear(&mut self) {
        self.buffer.clear();
        self.index = 0;
    }
}

impl Debug for IndexedMoveBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut lines = self
            .buffer
            .iter()
            .map(|m| format!("{:?}", m))
            .collect::<Vec<_>>();

        if self.index < self.buffer.len() {
            lines[self.index] = format!("{} <=========", lines[self.index]);
        }

        write!(f, "\n{}", &indent(&lines.join("\n"), 2))
    }
}

#[derive(Default, Debug)]
pub struct TraversalStackFrame<D> {
    pub game: Game,

    move_options: Option<MoveOptions>,
    pub danger: Option<Danger>,
    pub moves: StableOption<IndexedMoveBuffer>,

    pub history_move: Option<Move>,

    pub data: D,
}

impl<D: Debug> TraversalStackFrame<D> {
    pub fn setup_from_move(
        &mut self,
        previous: &mut TraversalStackFrame<D>,
        move_to_apply: &Move,
    ) -> ErrorResult<Legal> {
        self.game = previous.game;
        self.game.make_move(*move_to_apply)?;

        self.move_options = None;
        self.danger = None;
        self.moves.clear();
        self.history_move = Some(move_to_apply.clone());

        previous.lazily_generate_danger()?;

        if self
            .game
            .move_legality(move_to_apply, previous.danger.as_ref().as_result()?)
            == Legal::No
        {
            return Ok(Legal::No);
        }

        Ok(Legal::Yes)
    }

    pub fn setup_from_scratch(&mut self, game: Game) -> ErrorResult<()> {
        self.game = game;

        self.move_options = None;
        self.danger = None;
        self.moves.clear();

        self.lazily_generate_danger()?;
        Ok(())
    }

    pub fn set_move_options(&mut self, move_options: MoveOptions) -> ErrorResult<()> {
        if self.moves.is_some() {
            return err_result("cannot set move options after moves have been generated");
        }
        self.move_options = Some(move_options);
        Ok(())
    }

    pub fn last_future_move(&self) -> ErrorResult<Option<Move>> {
        if self.moves.is_some() {
            let moves = self.moves.get_ref().unwrap();
            if moves.index == 0 {
                return Ok(None);
            }
            Ok(moves.buffer.get(moves.index - 1).cloned())
        } else {
            err_result(&format!("no moves for {:#?}", self))?
        }
    }

    pub fn lazily_generate_moves<S: Fn(&Game, &mut Vec<Move>) -> ErrorResult<()>>(
        &mut self,
        move_sorter: S,
    ) -> ErrorResult<&IndexedMoveBuffer> {
        if self.moves.is_some() {
            return self.moves.get_ref().as_result();
        }

        self.moves.clear();
        self.moves.prepare_update();
        let buffer = &mut self.moves.get_mut().as_result()?.buffer;

        self.game.fill_pseudo_move_buffer(
            buffer,
            self.move_options.as_result()?,
        )?;

        move_sorter(&self.game, buffer)?;

        Ok(self.moves.get_ref().unwrap())
    }

    pub fn lazily_generate_danger(&mut self) -> ErrorResult<&Danger> {
        if self.danger.is_some() {
            return Ok(self.danger.as_ref().unwrap());
        }

        self.danger = Some(Danger::from(self.game.player(), self.game.bitboards())?);
        Ok(self.danger.as_ref().unwrap())
    }

    pub fn get_and_increment_move<S: Fn(&Game, &mut Vec<Move>) -> ErrorResult<()>>(
        &mut self,
        options: MoveOptions,
        move_sorter: S,
    ) -> ErrorResult<Option<Move>> {
        match self.move_options {
            Some(current_options) => {
                if current_options != options {
                    return err_result("move options changed");
                }
            }
            None => {
                self.set_move_options(options)?;
            }
        }

        self.lazily_generate_moves(move_sorter)?;

        let current_moves = self.moves.get_mut().as_result()?;
        if current_moves.index >= current_moves.buffer.len() {
            return Ok(None);
        }

        let m = current_moves.buffer.get(current_moves.index);
        current_moves.index += 1;

        Ok(m.cloned())
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
                move_options: None,
                danger: None,
                moves: StableOption::default(),
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

    pub fn get_and_increment_move<S: Fn(&Game, &mut Vec<Move>) -> ErrorResult<()>>(
        &mut self,
        options: MoveOptions,
        sorter: S,
    ) -> ErrorResult<Option<Move>> {
        let (current, _) = self.current_mut()?;
        current.get_and_increment_move(options, sorter)
    }
}

pub fn null_move_sort(_game: &Game, _moves: &mut Vec<Move>) -> ErrorResult<()> {
    Ok(())
}
