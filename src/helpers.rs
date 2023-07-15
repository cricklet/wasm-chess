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

pub fn flatten_arr_result<T>(
    result: ErrorResult<&[T]>,
) -> Box<dyn Iterator<Item = ErrorResult<&T>> + '_> {
    result.map_or_else(
        |err| -> Box<dyn Iterator<Item = ErrorResult<&T>> + '_> {
            let err_iter = iter::once(Err(err));
            let err_iter = Box::new(err_iter);
            err_iter
        },
        |t| -> Box<dyn Iterator<Item = ErrorResult<&T>> + '_> { Box::new(t.iter().map(Ok)) },
    )
}

pub fn flatten_iter_result<'t, T: 't>(
    result: ErrorResult<impl Iterator<Item = T> + 't>,
) -> Box<dyn Iterator<Item = ErrorResult<T>> + 't>
where
    T: Copy,
{
    let iter = result.unwrap();
    let result_iter = iter.map(|t| Ok(t));
    Box::new(result_iter)
}

pub fn map_successes<'t, T: 't, U>(
    results: impl Iterator<Item = ErrorResult<T>> + 't,
    callback: impl Fn(T) -> U + 't,
) -> impl Iterator<Item = ErrorResult<U>> + 't {
    results.map(move |result| result.map(&callback))
}
