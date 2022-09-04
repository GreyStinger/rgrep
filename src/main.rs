use std::{
    io::BufRead,
    path::PathBuf,
};

#[allow(dead_code)]
struct Config {
    pattern: String, 
    path: PathBuf,
}

impl Config {
    fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();

        // Get pattern from args
        let pattern = if args.len() < 2 {
            println!("Please pass rgrep a pattern to search for.");
            std::process::exit(0);
        } else {
            &args[1]
        };

        // Get path buffer from args
        let path: PathBuf = if args.contains(&"-i".to_string()) {
            let path_pos: usize = args.iter().position(|x| x.eq("-i")).unwrap() + 1;

            let buf: PathBuf = if path_pos > args.len() {
                std::env::current_dir().unwrap()
            } else {
                let mut buf: PathBuf = PathBuf::new();
                let path_str: &String = &args[path_pos];
                buf.push(path_str);
                buf
            };

            buf
        } else { 
            std::env::current_dir().unwrap()
        };
        

        Config {
            pattern: pattern.to_string(),
            path: path 
        }
    }
}

// fn scan_file(results :&Vec<usize, usize>) {

//     for val in results.iter() {
//         println!("{0}: {1}", val.0 + 1, stdin[val.0].trim());
//     }
// }

fn main() {
    let config = Config::new();

    // Results vector organized as: line_num: usize, line_col: usize, line: &str
    let mut results: Vec<(usize, usize, &str)> = Vec::new();

    let stdin: std::io::StdinLock = std::io::stdin().lock();
    let stdin: Vec<String> = stdin.lines().collect::<Result<Vec<String>, std::io::Error>>().unwrap();

    for value in stdin.iter().enumerate() {
        if let Some(index) = value.1.find(&config.pattern) {
            results.push((value.0, index, value.1.trim()));
        }
    }

    for val in results.iter() {
        println!("{0}: {1} : {2}", val.0 + 1, val.2, val.1);
    }
}
