use crate::helpers::{ErrorResult, OptionResult, StableOption};

pub trait Lazy<T> {
    fn get(&mut self) -> ErrorResult<&T>;
    fn get_mut(&mut self) -> ErrorResult<&mut T>;
}

struct LazyCallback<T> {
    value: StableOption<T>,
    callback: Box<dyn Fn(&mut T) -> ErrorResult<()>>,
}

impl<T: Default> LazyCallback<T> {
    fn new(callback: Box<dyn Fn(&mut T) -> ErrorResult<()>>) -> LazyCallback<T> {
        LazyCallback {
            value: StableOption::default(),
            callback,
        }
    }
}

impl<T> Lazy<T> for LazyCallback<T> {
    fn get(&mut self) -> ErrorResult<&T> {
        if self.value.is_none() {
            self.value.update(&mut |value| (self.callback)(value))?;
        }
        self.value.as_ref().as_result()
    }

    fn get_mut(&mut self) -> ErrorResult<&mut T> {
        if self.value.is_none() {
            self.value.update(&mut |value| (self.callback)(value))?;
        }
        self.value.as_mut().as_result()
    }
}

struct TestBar {
    danger: LazyCallback<i32>,
    moves: Vec<i32>,
}

struct TestFoo {
    current: TestBar,
}

#[test]
fn test_understand_multiple_mut_refs() {
    let mut f = TestFoo {
        current: TestBar {
            danger: LazyCallback::new(Box::new(|t| {
                *t += 1;
                Ok(())
            })),
            moves: vec![],
        },
    };

    {
        let f = &mut f;
        let current = &mut f.current;
        let danger = &mut current.danger;
        let moves = &mut current.moves;

        assert_eq!(*danger.get().unwrap(), 1);
        assert_eq!(*danger.get().unwrap(), 1);
        assert_eq!(*danger.get().unwrap(), 1);
        assert_eq!(*danger.get().unwrap(), 1);
        moves.push(1);
    }
}
