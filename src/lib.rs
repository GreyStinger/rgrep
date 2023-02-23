#![feature(proc_macro_hygiene)]

use clap::{arg, Arg, ArgMatches, Command};
use lazy_static::lazy_static;
use std::io::ErrorKind;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

lazy_static! {
    static ref ARGS: ArgMatches = Command::new("RGrep")
        .author("Jayden Andrews")
        .version("0.1.1")
        .about("A Grep like tool built in rust")
        .args([
            arg!(<PATTERN> "Pattern to search for"),
            Arg::new("PATH").required(false),
            Arg::new("no-exclusion")
                .long("no-exclusion")
                .help("Disable exclusion of certain file extensions")
                .required(false)
                .action(clap::ArgAction::SetTrue),
        ])
        .after_help("This tool is a replica of the grep tool originally \
            authored by Ken Thompson nearly 50 years ago using a newer \
            language and generally designed to be a bit more user friendly. \
            It's not yet quite as fast but it's a lot simpler to read and \
            understand so please check out its source code if you're interested.")
        .get_matches();
}

static mut T_RESULTS: usize = 0;

/// Searches for a pattern in a file or stdin and outputs matching lines to stdout.
pub fn run() {
    let mut c_writer = CustomWriter::new();

    // Decide what to do for either file or stdin
    let pattern: &str = ARGS.get_one::<String>("PATTERN").unwrap();
    match ARGS.get_one::<String>("PATH") {
        Some(path) => {
            process_directory(std::path::Path::new(&path), &mut c_writer, pattern);
        }
        None => {
            let stdin = std::io::stdin();
            c_writer
                .write_from_buff(&mut stdin.lock(), pattern)
                .unwrap();
        }
    }

    c_writer.print(&format!("\nTotal Results: {0}", unsafe { T_RESULTS }));

    c_writer.flush();
}

/// Recursively processes a directory, printing out the name of each file and running
/// write_from_buff on each file with a specified pattern.
///
/// # Arguments
///
/// * path - A std::path::Path reference to the directory to process.
/// * c_writer - A mutable reference to a CustomWriter instance.
/// * pattern - A string slice specifying the pattern to use when calling write_from_buff.
///
/// # Example
///
/// ```
/// use my_lib::CustomWriter;
/// use std::path::Path;
///
/// let path = Path::new("/path/to/directory");
/// let mut c_writer = CustomWriter::new();
/// let pattern = "pattern";
/// process_directory(path, &mut c_writer, pattern);
/// ```
fn process_directory(path: &std::path::Path, c_writer: &mut CustomWriter, pattern: &str) {
    #[cfg(debug_assertions)]
    c_writer.flush();
    // check if the path is a directory
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.is_dir() {
            // go through each file and directory recursively
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        process_directory(entry.path().as_path(), c_writer, pattern);
                    }
                }
            }
        } else {
            let name = match path.file_name() {
                Some(name) => name.to_str().unwrap_or("Err"),
                None => "Err",
            };
            c_writer.print(&format!("File: {}", name));
            let mut file_handler = BufReader::new(File::open(path).expect("Failed to open file"));
            match c_writer.write_from_buff(&mut file_handler, pattern) {
                Err(e) => {
                    if !(e.kind() == ErrorKind::InvalidData) {
                        eprintln!("Could not parse buffer: {}", e);
                    }
                }
                _ => {}
            };
        }
    }
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
    pub fn write_from_buff(
        &mut self,
        lines_buf: &mut impl std::io::BufRead,
        pattern: &str,
    ) -> Result<(), std::io::Error> {
        for line_result in lines_buf.lines() {
            #[cfg(feature = "no_color")]
            {
                let line = line_result?;
                
                if line.contains(pattern) {
                    self.print(&line);
                    unsafe {T_RESULTS += 1 };
                }
            }

            #[cfg(not(feature = "no_color"))]
            {
                let mut line = line_result?;
                let line_clone = line.clone();
                let matched_patterns: Vec<(usize, &str)> = line_clone.match_indices(pattern).collect();
                let mut matched_patterns: std::vec::IntoIter<(usize, &str)> =
                    matched_patterns.into_iter();

                if let Some(s_match) = matched_patterns.next() {
                    Self::color_piece(&mut line, s_match, &mut matched_patterns, 0);
                    self.print(&line);
                    unsafe { T_RESULTS += 1 };
                }
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "no_color"))]
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
        let start_char = s.chars().nth(current_match.0 + i_add - 1).unwrap_or(' ');

        let is_full =
            start_char == ' ' && (end_char == ' ' || end_char == '\0' || end_char == '\n');
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
