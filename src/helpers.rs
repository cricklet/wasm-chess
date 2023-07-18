use std::{backtrace::Backtrace, iter};

#[derive(Debug)]
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

pub type ErrorResult<T> = Result<T, Error>;

pub type VoidResult = Result<(), Error>;

pub fn err<T>(msg: &str) -> ErrorResult<T> {
    let backtrace = Backtrace::force_capture();
    let backtrace_str = backtrace.to_string();
    let mut backtrace_lines = backtrace_str.lines().collect::<Vec<_>>();

    let project_name = module_path!().split("::").next().unwrap();

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

    let backtrace_str = pretty_backtrace(&backtrace_lines);
    let backtrace_str = backtrace_str.unwrap();

    Err(Error {
        msg: msg.to_string() + "\n" + backtrace_str.as_str(),
    })
}

pub fn indent(input: &str, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let mut output = String::new();
    for line in input.lines() {
        output += &indent_str;
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
