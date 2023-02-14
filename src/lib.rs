use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

/// Searches for a pattern in a file or stdin and outputs matching lines to stdout.
///
/// # Examples
///
/// To search for the pattern `foo` in a file:
///
/// ```
/// rgrep::run_with_args(vec!["rgrep", "foo", "path/to/file.txt"]);
/// ```
///
/// To search for the pattern `bar` in stdin:
///
/// ```
/// rgrep::run_with_args(vec!["rgrep", "bar"]);
/// ```
pub fn run() {
    // Get args and look for pattern
    let args: std::vec::Vec<String> = std::env::args().collect();

	run_with_args(&args);
}

fn run_with_args(args: &std::vec::Vec<String>) {
    let mut c_writer = CustomWriter::new();

    // This closure, `usage`, is used to print an error message indicating that the user
    // has used the rgrep command incorrectly, and then terminate the program with an exit code of 1.
    let usage = || {
        print!("Bad Usage\n");
        print!("Use: <command> | rgrep <pattern> or rgrep <pattern> <dir/file>\n");
        std::process::exit(1);
    };

    if args.len() < 2 {
        usage();
    }
    let pattern = match args.get(1) {
        Some(pattern) => pattern,
        None => usage(),
    };

    // Decide what to do for either file or stdin
    match args.get(2) {
        Some(fname) => {
            // TODO: Add better error handling for the file opening
            let mut file_handler = BufReader::new(File::open(fname).expect("Failed to open file"));
            c_writer.write_from_buff(&mut file_handler, &pattern)
        }
        None => {
            let stdin = std::io::stdin();
            c_writer.write_from_buff(&mut stdin.lock(), &pattern);
        }
    }

    c_writer.flush();
}

pub struct CustomWriter<'a> {
    stdout_writer: std::io::BufWriter<std::io::StdoutLock<'a>>,
}

impl<'a> CustomWriter<'_> {
    pub fn new() -> Self {
        let stdout = std::io::stdout();
        let stdout_lock = stdout.lock();
        let stdout_writer = std::io::BufWriter::new(stdout_lock);

        Self { stdout_writer }
    }

    /// Writes lines from a buffer to standard output, highlighting any occurrences of a given
    /// pattern with ANSI color codes.
    ///
    /// # Arguments
    ///
    /// * `lines_buf` - A mutable reference to a buffer of lines to write.
    /// * `pattern` - A string slice with the pattern to search for in each line.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use rgrep::CustomWriter;
    ///
    /// let mut buffer = Cursor::new("hello\nworld\n");
    /// let mut writer = CustomWriter::new();
    /// writer.write_from_buff(&mut buffer, "l");
    /// ```
    pub fn write_from_buff(&mut self, lines_buf: &mut impl std::io::BufRead, pattern: &str) {
        let mut results: usize = 0;

        for line_result in lines_buf.lines() {
            let mut line = match line_result {
                Ok(line) => line,
                Err(e) => {
                    eprintln!("Could not parse buffer: {}", e);
                    continue;
                }
            };

            let line_clone = line.clone();
            let matched_patterns: Vec<(usize, &str)> = line_clone.match_indices(pattern).collect();
            let mut matched_patterns: std::vec::IntoIter<(usize, &str)> =
                matched_patterns.into_iter();

            if let Some(s_match) = matched_patterns.next() {
                Self::color_piece(&mut line, s_match, &mut matched_patterns, 0);
                self.print(&line);
                results += 1;
            }
        }

        self.print(&format!("\nTotal Results: {0}", results));
    }

    /// Recursively applies color codes to a string based on matches of a given pattern.
    ///
    /// # Arguments
    ///
    /// * `s` - A mutable reference to the string to apply color codes to.
    /// * `current_match` - A tuple representing the starting index and the matched string for the current match.
    /// * `matches` - A mutable iterator over the remaining matches.
    /// * `i_add` - The index to add to the starting index for each match. This is used for recursive calls.
    ///
    /// # Example
    ///
    /// ```
    /// let mut s = String::from("hello world");
    /// let matches = vec![(0, "h"), (6, "w")];
    /// let mut matches_iter = matches.into_iter();
    /// color_piece(&mut s, matches_iter.next().unwrap(), &mut matches_iter, 0);
    /// assert_eq!(s, "\u{1b}[31mh\u{1b}[0mello \u{1b}[32mw\u{1b}[0morld");
    /// ```
    fn color_piece(
        s: &mut String,
        current_match: (usize, &str),
        matches: &mut std::vec::IntoIter<(usize, &str)>,
        i_add: usize,
    ) {
        let end_char = s
            .chars()
            .nth(current_match.0 + current_match.1.len() + i_add)
            .unwrap_or(' ');
        let is_full = s.chars().nth(current_match.0 + i_add - 1).unwrap() == ' '
            && (end_char == ' ' || end_char == '\0' || end_char == '\n');
        let color_code = if is_full { "\u{1b}[32m" } else { "\u{1b}[31m" };
        let reset_code = "\u{1b}[0m";
        s.insert_str(current_match.0 + i_add, color_code);
        s.insert_str(
            current_match.0 + current_match.1.len() + i_add + color_code.len(),
            reset_code,
        );
        matches
            .next()
            .map(|next_match| Self::color_piece(s, next_match, matches, i_add + 9));
    }

    /// Writes a message to the wrapped writer with a newline character.
    ///
    /// # Arguments
    ///
    /// * `msg` - A string slice to be written to the writer.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Write;
    /// use crate::CustomWriter;
    ///
    /// let mut writer = CustomWriter::new();
    /// writer.print("Hello, world!");
    /// ```
    pub fn print(&mut self, msg: &str) {
        if let Err(e) = writeln!(self.stdout_writer, "{}", msg) {
            eprintln!("pipeline::print failed: {}", e);
        }
    }

    /// Flushes the underlying writer, ensuring that all buffered data has been written.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Write;
    /// use crate::CustomWriter;
    ///
    /// let mut writer = CustomWriter::new();
    /// writeln!(writer, "Hello, world!");
    /// writer.flush();
    /// ```
    pub fn flush(&mut self) {
        if let Err(e) = self.stdout_writer.flush() {
            eprintln!("pipeline::CustomWriter::flush failed: {:?}", e);
        }
    }
}
