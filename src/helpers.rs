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

pub fn iter_results_from_result_iter<'iter, T: 'iter>(
    result_iter: ErrorResult<impl Iterator<Item = T> + 'iter>,
) -> Box<dyn Iterator<Item = ErrorResult<T>> + 'iter>
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

pub fn map_successes<'iter, T: 'iter, U>(
    results: impl Iterator<Item = ErrorResult<T>> + 'iter,
    callback: impl Fn(T) -> U + 'iter,
) -> impl Iterator<Item = ErrorResult<U>> + 'iter {
    results.map(move |result| result.map(&callback))
}
