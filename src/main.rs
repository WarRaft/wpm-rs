//! `wpm-rs`: checks pathing maps and describes them. See `usage`.

use std::process::ExitCode;

use wpm_rs::{FLAGS, parse};

fn usage() {
    eprintln!("usage: wpm-rs <war3map.wpm>          check: read and write back");
    eprintln!("       wpm-rs --info <war3map.wpm>   the size, and how many cells have each flag");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (info, path) = match args.as_slice() {
        ["--info", path] => (true, *path),
        [path] if !path.starts_with('-') => (false, *path),
        _ => {
            usage();
            return ExitCode::FAILURE;
        }
    };
    let result = std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| parse(&bytes).map(|m| (bytes, m)).map_err(|e| e.to_string()));
    let (bytes, m) = match result {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !info {
        if m.to_bytes().ok() != Some(bytes) {
            eprintln!("{path}: differs after writing");
            return ExitCode::FAILURE;
        }
        println!("{path}: {}x{} cells", m.width, m.height);
        return ExitCode::SUCCESS;
    }
    println!("cells\t{}x{}", m.width, m.height);
    for (bit, name) in FLAGS {
        let n = m.cells.iter().filter(|&&c| c & bit != 0).count();
        println!("0x{bit:02X}\t{name}\t{n}");
    }
    ExitCode::SUCCESS
}
