use std::error::Error;

use redis::Commands;
use serde::{Deserialize, Serialize};

use crate::cache;
mod pathfinding;

pub fn find(source_name: &str, target_name: &str) -> Result<(), Box<dyn Error>> {
    let mut cache = redis::Client::open("redis://127.0.0.1/")?.get_connection()?;

    println!("Connected to cache");
    println!("Starting search from {source_name} to {target_name}");
    let start = std::time::Instant::now();

    let traversal = traverse(&mut cache, source_name, target_name);

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

pub fn find_json(source_name: &str, target_name: &str) {
    let mut cache = redis::Client::open("redis://127.0.0.1/")
        .unwrap()
        .get_connection()
        .unwrap();
    let traversal = traverse(&mut cache, source_name, target_name);
    let json = serde_json::to_string(&traversal).unwrap();
    println!("{json}");
}

#[derive(Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
}
pub type TraverseResult = Result<Vec<Page>, String>;

pub fn traverse(
    cache: &mut redis::Connection,
    source_name: &str,
    target_name: &str,
) -> TraverseResult {
    let target_id: String = cache
        .get(cache::IdOf(target_name))
        .map_err(|_| "Could not find target page")?;
    let source_id: String = cache
        .get(cache::IdOf(source_name))
        .map_err(|_| "Could not find source page")?;

    let result = pathfinding::bfs(
        &source_id,
        |id| {
            let link_ids: Vec<String> = cache.smembers(cache::Links(id)).unwrap();
            link_ids
        },
        |id| id == &target_id,
    );

    match result {
        Some(page_ids) => {
            let names: Vec<String> = cache.mget(&page_ids).unwrap();
            let page_aliases: Vec<Vec<String>> = page_ids
                .iter()
                .map(|id| {
                    let alias_ids: Vec<String> = cache.smembers(cache::Aliases(id)).unwrap();
                    if alias_ids.is_empty() {
                        vec![]
                    } else {
                        cache.mget(&alias_ids).unwrap()
                    }
                })
                .collect();
            let pages = names
                .iter()
                .zip(page_ids.iter())
                .zip(page_aliases.iter())
                .map(|((name, id), aliases)| Page {
                    id: id.to_string(),
                    name: name.to_string(),
                    aliases: aliases.clone(),
                })
                .collect();
            Ok(pages)
        }
        None => Err("No path found".to_string()),
    }
}
