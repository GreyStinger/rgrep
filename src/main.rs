mod rgrep {
    use std::{io::{BufRead, Write, BufReader}, fs::File};

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
                let mut file_handler = BufReader::new(File::open(fname).expect("Failed to open file"));
                c_writer.write_from_buff(&mut file_handler, &pattern)

            },
            None => {
                let stdin = std::io::stdin();
                c_writer.write_from_buff(&mut stdin.lock() as &mut dyn BufRead, &pattern);
            },
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

        fn write_from_buff(&mut self, lines_buf: &mut dyn std::io::BufRead, pattern: &String) {
            let mut results = 0;

            for line in lines_buf.lines() {
                let mut line = line.expect("Could not parse buffer");
                let line_clone = line.clone();
                let matched_patterns: Vec<(usize, &str)> = line_clone.match_indices(pattern).collect();
                let mut matched_patterns: std::vec::IntoIter<(usize, &str)> = matched_patterns.into_iter();

                if let Some(s_match) = matched_patterns.next() {
                    Self::color_piece_s(&mut line, s_match, &mut matched_patterns);
                    self.print(&line);
                    results += 1;
                }
            }

            self.print(&format!("\nTotal Results: {0}", results));
        }

        /// A start function for color_piece
        fn color_piece_s(
            s: &mut String,
            current_match: (usize, &'a str),
            matches: &mut std::vec::IntoIter<(usize, &str)>,
        ) {
            Self::color_piece(s, current_match, matches, 0);
        }

        /// Color the piece that matches the pattern
        fn color_piece(
            s: &mut String,
            current_match: (usize, &'a str),
            matches: &mut std::vec::IntoIter<(usize, &str)>,
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

fn main() {rgrep::run();}
