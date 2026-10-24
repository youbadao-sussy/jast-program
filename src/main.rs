use jast_program::lexer::lex;
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, cmd, path] if cmd == "lex" => cmd_lex(path),
        _ => {
            eprintln!("usage: jastc lex <file.jp>");
            process::exit(2);
        }
    }
}

fn cmd_lex(path: &str) {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{path}: {e}");
            process::exit(1);
        }
    };
    let out = lex(&src);
    for t in &out.tokens {
        println!("{}:{}\t{}", t.span.line, t.span.col, t.kind);
    }
    if !out.errors.is_empty() {
        for e in &out.errors {
            eprintln!("{path}:{e}");
        }
        process::exit(1);
    }
}