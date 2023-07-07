pub type ErrorResult<T> = Result<T, String>;

pub type VoidResult = Result<(), String>;

pub enum Loop {
    Continue,
    Break,
}

pub struct IteratorFn<T, F>
where
    F: FnMut() -> Option<T>,
{
    pub callback: F,
}

impl<T, F> IteratorFn<T, F>
where
    F: FnMut() -> Option<T>,
{
    pub fn new(callback: F) -> IteratorFn<T, F> {
        IteratorFn { callback }
    }
}

impl<T, F> Iterator for IteratorFn<T, F>
where
    F: FnMut() -> Option<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        (self.callback)()
    }
}
