mod pipeline {
    use std::io::{BufRead, Write};

    pub struct Parser<'a> {
        p_stdin: std::io::Stdin,
        c_writer: CustomWriter<'a>
    }

    impl Parser<'_> {
        pub fn new() -> Self {
            Self {
                p_stdin: std::io::stdin(),
                c_writer: CustomWriter::new()
            }
        }

        /// Takes the input from the command line and parses it
        pub fn parse(&mut self, pattern: &String) {
            for line in self.p_stdin.lock().lines() {
                let line = line.expect("Could not parse stdin");
                if line.contains(pattern) {
                    self.c_writer.print(&line);
                }
            };
        }

        pub fn close(&mut self) {
            self.c_writer.close();
        }
    }


    struct CustomWriter<'a> {
        p_writer: std::io::BufWriter<std::io::StdoutLock<'a>>
    }

    impl CustomWriter<'_> {
        pub fn new() -> Self {
            let p_stdout_lock = std::io::stdout().lock();

            Self {
                p_writer: std::io::BufWriter::new(p_stdout_lock)
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
                println!("Pipleline::flush failed");
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
    if args.len() < 2 { usage(); }
    let pattern = match args.nth(1) {
        Some(pattern) => pattern,
        None => usage()
    };

    let mut pipe_handler = pipeline::Parser::new();
    pipe_handler.parse(&pattern);
    pipe_handler.close();
}
