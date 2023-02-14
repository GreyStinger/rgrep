mod rgrep {
    use std::{
        fs::File,
        io::{BufRead, BufReader, Write},
    };

    pub fn run() {
        let mut c_writer = CustomWriter::new();

        let usage = || {
            print!("Bad Usage\n");
            print!("Use: <command> | rgrep <pattern> or rgrep <pattern> <dir/file>\n");
            std::process::exit(1);
        };

        // Get args and look for pattern
        let mut args = std::env::args();
        if args.len() < 2 {
            usage();
        }
        let pattern = match args.nth(1) {
            Some(pattern) => pattern,
            None => usage(),
        };

        // Decide what to do for either file or stdin
        match args.next() {
            Some(fname) => {
                // TODO: Add better error handling for the file opening
                let mut file_handler =
                    BufReader::new(File::open(fname).expect("Failed to open file"));
                c_writer.write_from_buff(&mut file_handler, &pattern)
            }
            None => {
                let stdin = std::io::stdin();
                c_writer.write_from_buff(&mut stdin.lock() as &mut dyn BufRead, &pattern);
            }
        }

        c_writer.flush();
    }

    struct CustomWriter<'a> {
        stdout_writer: std::io::BufWriter<std::io::StdoutLock<'a>>,
    }

    impl<'a> CustomWriter<'_> {
        pub fn new() -> Self {
            let p_stdout_lock = std::io::stdout().lock();

            Self {
                stdout_writer: std::io::BufWriter::new(p_stdout_lock),
            }
        }

        fn write_from_buff(&mut self, lines_buf: &mut dyn std::io::BufRead, pattern: &str) {
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
                let matched_patterns: Vec<(usize, &str)> =
                    line_clone.match_indices(pattern).collect();
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

        /// For each match of the pattern in the string, adds terminal color codes
        /// before and after the match. The exact color code used depends on whether
        /// the match is surrounded by whitespace or not.
        ///
        /// # Arguments
        ///
        /// * `s` - A mutable string to modify.
        /// * `current_match` - A tuple containing the starting index and the matched substring.
        /// * `matches` - An iterator over the remaining matches in the string.
        /// * `i_add` - The number of characters added to `s` so far.
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

        /// Custom higher performance print system for efficient
        /// buffered printing
        pub fn print(&mut self, msg: &String) {
            if let Err(e) = write!(self.stdout_writer, "{0}\n", msg) {
                println!("pipeline::print failed");
                dbg!(e);
            }
        }

        pub fn flush(&mut self) {
            if let Err(e) = self.stdout_writer.flush() {
                println!("pipeline::CustomWriter::flush failed");
                dbg!(e);
            }
        }
    }
}

fn main() {
    rgrep::run();
}
