mod rgrep {
    use std::io::{BufRead, Write};

    pub struct StdInParser<'a> {
        p_stdin: std::io::Stdin,
        c_writer: CustomWriter<'a>,
    }

    impl StdInParser<'_> {
        pub fn new() -> Self {
            Self {
                p_stdin: std::io::stdin(),
                c_writer: CustomWriter::new(),
            }
        }

        /// Takes the input from the command line and parses it
        pub fn parse(&mut self, pattern: &String) {
            let mut results = 0;

            for line in self.p_stdin.lock().lines() {
                let mut line = line.expect("Could not parse stdin");
                let line_clone = line.clone();
                let mut matched_patterns = line_clone.match_indices(pattern);

                if let Some(s_match) = matched_patterns.next() {
                    CustomWriter::color_piece_s(&mut line, s_match, &mut matched_patterns);
                    self.c_writer.print(&line);
                    results += 1;
                }
            }

            self.c_writer
                .print(&format!("\nTotal Results: {0}", results));
        }

        pub fn close(&mut self) {
            self.c_writer.close();
        }
    }

    struct CustomWriter<'a> {
        p_writer: std::io::BufWriter<std::io::StdoutLock<'a>>,
    }

    impl<'a> CustomWriter<'_> {
        pub fn new() -> Self {
            let p_stdout_lock = std::io::stdout().lock();

            Self {
                p_writer: std::io::BufWriter::new(p_stdout_lock),
            }
        }

        fn write_from_buff() {
            todo!();
        }

        /// A start function for color_piece
        fn color_piece_s(
            s: &mut String,
            current_match: (usize, &'a str),
            matches: &mut std::str::MatchIndices<&String>,
        ) {
            Self::color_piece(s, current_match, matches, 0);
        }

        /// Color the piece that matches the pattern
        fn color_piece(
            s: &mut String,
            current_match: (usize, &'a str),
            matches: &mut std::str::MatchIndices<&String>,
            i_add: usize,
        ) {
            let end_char = match s
                .chars()
                .nth(current_match.0 + current_match.1.len() + i_add)
            {
                Some(end_char) => end_char,
                None => ' ',
            };

            let is_full = s.chars().nth(current_match.0 + i_add - 1).unwrap() == ' '
                && (end_char == ' ' || end_char == '\0' || end_char == '\n');

            s.insert_str(current_match.0 + current_match.1.len() + i_add, "\u{1b}[0m");

            if is_full {
                s.insert_str(current_match.0 + i_add, "\u{1b}[32m");
            } else {
                s.insert_str(current_match.0 + i_add, "\u{1b}[31m");
            }

            if let Some(s_match) = matches.next() {
                let i_add = i_add + 9; // 9 works, I though it was 15 but idk
                Self::color_piece(s, s_match, matches, i_add);
            }
        }

        /// Custom higher performance print system for efficient
        /// buffered printing
        pub fn print(&mut self, msg: &String) {
            if let Err(e) = write!(self.p_writer, "{0}\n", msg) {
                println!("pipeline::print failed");
                dbg!(e);
            }
        }

        pub fn flush(&mut self) {
            if let Err(e) = self.p_writer.flush() {
                println!("pipeline::CustomWriter::flush failed");
                dbg!(e);
            }
        }

        pub fn close(&mut self) {
            self.flush();
        }
    }
}

fn main() {
    let usage = || {
        print!("Missing pattern.\n");
        print!("Please use it as: <command> | rgrep <pattern>\n");
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

    let mut pipe_handler = rgrep::StdInParser::new();
    pipe_handler.parse(&pattern);
    pipe_handler.close();
}
