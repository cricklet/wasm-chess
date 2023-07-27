use std::{backtrace::Backtrace, iter};

pub struct Error {
    pub msg: String,
}

impl Error {
    pub fn throw<T>(&self) -> T {
        panic!("{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\n{}", indent(self.msg.as_str(), 2))
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {{\n{}}}", indent(self.msg.trim(), 2))
    }
}

pub type ErrorResult<T> = Result<T, Error>;

pub type VoidResult = Result<(), Error>;

pub fn err(msg: &str) -> Error {
    let backtrace = Backtrace::force_capture();
    let backtrace_str = backtrace.to_string();
    let mut backtrace_lines = backtrace_str.lines().collect::<Vec<_>>();

    let project_name = module_path!().split("::").next().unwrap();

    let first_line_without_helpers_err = {
        let mut first_line_without_helpers_err = None;
        for (i, line) in backtrace_lines.iter().enumerate() {
            if line.contains("helpers::err") {
                first_line_without_helpers_err = Some(i);
                break;
            }
        }
        first_line_without_helpers_err
    };

    let last_line_with_project_name = {
        let mut last_line_with_project_name = None;
        for (i, line) in backtrace_lines.iter().enumerate() {
            if line.contains(project_name) {
                last_line_with_project_name = Some(i);
            }
        }
        last_line_with_project_name
    };

    if let Some(i) = last_line_with_project_name {
        backtrace_lines = backtrace_lines[..i + 1].to_vec();
    }

    if let Some(i) = first_line_without_helpers_err {
        if i + 2 < backtrace_lines.len() {
            backtrace_lines = backtrace_lines[i + 2..].to_vec();
        }
    }

    let backtrace_str = pretty_backtrace(&backtrace_lines);
    let backtrace_str = backtrace_str.unwrap();

    Error {
        msg: msg.to_string() + "\n" + backtrace_str.as_str(),
    }
}

pub fn err_result<T>(msg: &str) -> ErrorResult<T> {
    Err(err(msg))
}

pub fn indent(input: &str, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    prefix(input, &indent_str)
}

pub fn prefix(input: &str, p: &str) -> String {
    let mut output = String::new();
    for line in input.lines() {
        output += p;
        output += line;
        output += "\n";
    }
    output
}

pub fn split_into_rows_and_cols<'line, LineIter>(
    lines: LineIter,
    num_columns: usize,
) -> Vec<Vec<&'line str>>
where
    LineIter: Iterator<Item = &'line str>,
{
    let mut rows = vec![];

    for (i, line) in lines.enumerate() {
        let row = i / num_columns;

        if rows.len() <= row {
            rows.push(vec![]);
        }
        rows[row].push(line);
    }

    rows
}

pub fn display_rows_and_columns(rows_and_columns: &Vec<Vec<&str>>) -> ErrorResult<String> {
    let num_columns = rows_and_columns[0].len();

    let mut col_sizes = vec![0; num_columns];
    for row in rows_and_columns {
        if row.len() > num_columns {
            return Err(Error {
                msg: format!("Expected {} columns, got {}", num_columns, row.len()),
            });
        }

        for (col, value) in row.iter().enumerate() {
            col_sizes[col] = col_sizes[col].max(value.len());
        }
    }

    let mut output = String::new();
    for row in rows_and_columns {
        for value in row {
            output += value;
            output += &" ".repeat(col_sizes[0] - value.len());
            output += " ";
        }
        output += "\n";
    }

    Ok(output)
}

pub fn pretty_backtrace(lines: &Vec<&str>) -> ErrorResult<String> {
    let num_columns = 2;
    let num_rows = (lines.len() as f32 / num_columns as f32).ceil() as usize;

    let lines = lines.iter().map(|&line| line.trim());

    let rows_and_cols = split_into_rows_and_cols(lines, num_columns);
    if rows_and_cols.len() != num_rows {
        return Err(Error {
            msg: format!("Expected {} rows, got {}", num_rows, rows_and_cols.len()),
        });
    }

    // let rows_and_cols = rows_and_cols
    //     .into_iter()
    //     .map(|row| row.into_iter().rev().collect())
    //     .collect();

    display_rows_and_columns(&rows_and_cols)
}

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

struct RecursiveStrings {
    current: String,
    next: Box<dyn Iterator<Item = RecursiveStrings>>,
}

impl RecursiveStrings {
    fn new() -> RecursiveStrings {
        RecursiveStrings {
            current: "x".to_string(),
            next: Box::new(std::iter::empty()),
        }
    }

    fn traverse(self) -> Box<dyn Iterator<Item = String>> {
        let once = std::iter::once(self.current);
        let future = self.next.map(|s| s.traverse());
        let future = future.flatten();

        let all = once.chain(future);

        Box::new(all)
    }
}

#[test]
fn test_understand_traversal_iter_string() {
    let rec = RecursiveStrings::new();
    for _ in rec.traverse() {}
}

fn add_iter(x: i32) -> Box<dyn Iterator<Item = i32>> {
    // fails because x will go out of scope
    // Box::new((0..).map(|i| i + x))

    // works because x is moved into the closure
    Box::new((0..).map(move |i| i + x).take(x as usize))
}

#[test]
fn test_understand_iter_from_params() {
    let x = add_iter(5);
    for _ in x {}
}

pub fn debug_and_return<T>(t: T) -> T
where
    T: std::fmt::Debug,
{
    println!("{:#?}", t);
    t
}

pub fn display_and_return<T>(t: T) -> T
where
    T: std::fmt::Display,
{
    println!("{}", t);
    t
}
