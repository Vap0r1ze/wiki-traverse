#![feature(array_methods, int_roundings, iter_next_chunk, macro_metavar_expr)]

use std::{env, error::Error};

mod cache;

mod build;
mod traverse;

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let dumps_path = env::var("WIKI_DUMPS_PATH")?;
    let dump_timestamp = env::var("WIKI_DUMP_TIMESTAMP")?;

    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_, cmd] if cmd == "build" => build::build_graph(&dumps_path, &dump_timestamp)?,
        [_, cmd, source, target] if cmd == "find" => traverse::find(source, target)?,
        [_, cmd, source, target] if cmd == "find_many" => traverse::find_many(source, target)?,
        [_, cmd, source, target] if cmd == "json" => traverse::find_json(source, target),
        _ => {
            eprintln!("Usage: wiki-graph build");
            eprintln!("       wiki-graph find <source> <target>");
            eprintln!("       wiki-graph json <source> <target>");
            std::process::exit(1);
        }
    }

    Ok(())
}
