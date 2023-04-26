use std::error::Error;

use self::pathfinding::{Page, PathIterator};
mod pathfinding;

pub fn find(source_name: &str, target_name: &str) -> Result<(), Box<dyn Error>> {
    let cache = redis::Client::open("redis://127.0.0.1/")?.get_connection()?;

    println!("Connected to cache");
    println!("Starting search from {source_name} to {target_name}");
    let start = std::time::Instant::now();

    let traversal = traverse(source_name, target_name, cache);

    println!("Search took {:?}", start.elapsed());

    match traversal {
        Ok(pages) => {
            println!(
                "Path: {}",
                pages
                    .iter()
                    .map(|page| page.name.clone())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            );
        }
        Err(err) => {
            println!("No path found: {err:?}");
        }
    };

    Ok(())
}

pub fn find_many(source_name: &str, target_name: &str) -> Result<(), Box<dyn Error>> {
    let cache = redis::Client::open("redis://127.0.0.1/")?.get_connection()?;

    println!("Connected to cache");
    println!("Starting search from {source_name} to {target_name}");
    let start = std::time::Instant::now();

    let path_iter = PathIterator::new(source_name, target_name, cache)?;

    for path in path_iter {
        println!("[{:?}] Path: {}", start.elapsed(), path);
    }

    println!("Search took {:?}", start.elapsed());

    Ok(())
}

pub fn find_json_many(source_name: &str, target_name: &str) {
    let cache = redis::Client::open("redis://127.0.0.1/")
        .unwrap()
        .get_connection()
        .unwrap();

    let path_iter = match PathIterator::new(source_name, target_name, cache) {
        Ok(iter) => iter,
        Err(err) => {
            let err: Result<(), _> = Err(err);
            let json_err = serde_json::to_string(&err).unwrap();
            println!("{json_err}");
            return;
        }
    };
    for path in path_iter {
        let json = serde_json::to_string(&path.take_inner()).unwrap();
        println!("{json}");
    }
}

pub fn find_json(source_name: &str, target_name: &str) {
    let cache = redis::Client::open("redis://127.0.0.1/")
        .unwrap()
        .get_connection()
        .unwrap();
    let traversal = traverse(source_name, target_name, cache);
    let json = serde_json::to_string(&traversal).unwrap();
    println!("{json}");
}

fn traverse(
    source_name: &str,
    target_name: &str,
    cache: redis::Connection,
) -> Result<Vec<Page>, String> {
    PathIterator::new(source_name, target_name, cache)?
        .next()
        .map(|p| p.take_inner())
        .ok_or("No path found".to_string())
}
