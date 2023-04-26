use std::error::Error;

use worker_pool::{DownMsg, WorkerPool};

#[macro_use]
mod parser;
mod eater;
mod progress;
mod reader;

static REDIS_ADDR: &str = "redis://127.0.0.1/";
const THREAD_COUNT: usize = 8;
const CHUNK_SIZE: usize = THREAD_COUNT;

pub fn build_graph(dumps_path: &str, dump_timestamp: &str) -> Result<(), Box<dyn Error>> {
    let pages_path = format!("{}/enwiki-{}-page.sql", dumps_path, dump_timestamp);
    let redirects_path = format!("{}/enwiki-{}-redirect.sql", dumps_path, dump_timestamp);
    let pagelinks_path = format!("{}/enwiki-{}-pagelinks.sql", dumps_path, dump_timestamp);

    handle_dump(eater::pages_line, &pages_path, "page")?;
    handle_dump(eater::redirects_line, &redirects_path, "redirect")?;
    handle_dump(eater::pagelinks_line, &pagelinks_path, "pagelinks")?;

    Ok(())
}

pub fn handle_dump<F>(handler: F, dump_path: &str, name: &str) -> Result<(), Box<dyn Error>>
where
    F: Fn(&mut redis::Connection, String) + Clone + Sync + Send + 'static,
{
    let mut reader = reader::TableReader::new(dump_path, name)?;
    reader.index_lines();

    let mut pool: WorkerPool<(), String> = WorkerPool::new(CHUNK_SIZE);

    println!("Created thread pool with buffer size of {CHUNK_SIZE}");

    pool.execute_many(THREAD_COUNT, move |tx, rx| {
        let cache_client =
            redis::Client::open(REDIS_ADDR).expect("Tried opening a redis connection");
        let mut cache = cache_client
            .get_connection()
            .expect("Tried getting a redis connection");

        for msg in rx {
            let line = match msg {
                DownMsg::Other(line) => line,
                DownMsg::Stop => break,
                DownMsg::Continue => continue,
                DownMsg::Pause => continue,
            };

            handler(&mut cache, line);
            tx.send(()).expect("Tried sending OK to the pool");
        }
    });

    println!("Spawned {THREAD_COUNT} threads");

    let pb = progress::create(reader.len().div_ceil(CHUNK_SIZE as u64), false);
    pb.set_message(format!("Parsing `{name}` dump"));

    loop {
        let chunk = match reader.next_chunk::<CHUNK_SIZE>() {
            Ok(chunk) => chunk.to_vec(),
            Err(iter) => iter.collect(),
        };

        let size = chunk.len();

        if chunk.is_empty() {
            break;
        }

        for line in chunk {
            pool.broadcast_one(DownMsg::Other(line))
                .expect("Tried broadcasting a line to the pool");
        }

        for _ in 0..size {
            pool.recv()
                .expect("Tried waiting for a line to be processed");
        }

        pb.inc(1);
    }

    pb.set_message("Finishing up");
    let _ = pool.stop_and_join();
    pb.finish_with_message("Done!");

    Ok(())
}
