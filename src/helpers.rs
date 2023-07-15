use std::iter;

pub type ErrorResult<T> = Result<T, String>;

pub type VoidResult = Result<(), String>;

pub enum Loop {
    Continue,
    Break,
}

pub struct FnIterator<T, F>
where
    F: FnMut() -> Option<T>,
{
    pub next: F,
}

impl<T, F> FnIterator<T, F>
where
    F: FnMut() -> Option<T>,
{
    pub fn new(callback: F) -> FnIterator<T, F> {
        FnIterator { next: callback }
    }
}

impl<T, F> Iterator for FnIterator<T, F>
where
    F: FnMut() -> Option<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        (self.next)()
    }
}

pub fn iter_results_from_result_iter<'t, T: 't>(
    result_iter: ErrorResult<impl Iterator<Item = T> + 't>,
) -> Box<dyn Iterator<Item = ErrorResult<T>> + 't>
where
    T: Copy,
{
    let iter = match result_iter {
        Err(err) => return Box::new(iter::once(Err(err))),
        Ok(iter) => iter,
    };
    let result_iter = iter.map(|t| Ok(t));
    Box::new(result_iter)
}

pub fn map_successes<'t, T: 't, U>(
    results: impl Iterator<Item = ErrorResult<T>> + 't,
    callback: impl Fn(T) -> U + 't,
) -> impl Iterator<Item = ErrorResult<U>> + 't {
    results.map(move |result| result.map(&callback))
}
