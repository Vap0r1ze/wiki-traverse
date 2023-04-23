#![feature(array_methods, int_roundings, iter_next_chunk, macro_metavar_expr)]

use std::{env, error::Error};

mod cache;

mod build;
mod traverse;

fn unwrap_or_exit<T, E: std::fmt::Display>(opt: Option<T>, err: E) -> T {
    match opt {
        Some(value) => value,
        None => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let dumps_path = env::var("WIKI_DUMPS_PATH")?;
    let dump_timestamp = env::var("WIKI_DUMP_TIMESTAMP")?;

    let args: Vec<String> = env::args().collect();

    let cmd = unwrap_or_exit(args.get(1), "No command provided");

    if cmd == "build" {
        build::build_graph(&dumps_path, &dump_timestamp)?
    } else if cmd == "find" {
        let [source, target] = unwrap_or_exit(args.get(2..4), "No source or target provided") else {
            panic!()
        };
        traverse::find(source, target)?;
    } else if cmd == "json" {
        let [source, target] = unwrap_or_exit(args.get(2..4), "No source or target provided") else {
            panic!()
        };
        traverse::find_json(source, target);
    }

    Ok(())
}
