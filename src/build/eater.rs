use super::parser;
use crate::cache;
use redis::{Commands, Pipeline};

pub fn pages_line(cache: &mut redis::Connection, line: String) {
    let (_, rows) = parser::rows(parser::page_row)(&line).unwrap();

    let rows = rows
        .iter()
        .filter_map(|(id, ns, name, _, _, _, _, _, _, _, _, _, _)| {
            if *ns == "0" {
                Some((*id, parser::string_value(name)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let lookups: Vec<(String, &str)> = rows
        .iter()
        .map(|(id, name)| (format!("idof:{name}"), *id))
        .collect::<Vec<_>>();

    if !rows.is_empty() {
        let _: () = cache.mset(&rows).expect("Tried `mset`ing the page rows");
        let _: () = cache
            .mset(&lookups)
            .expect("Tried `mset`ing the page lookups");
    }
}

pub fn redirects_line(cache: &mut redis::Connection, line: String) {
    let (_, rows) = parser::rows(parser::redirect_row)(&line).unwrap();

    let rows = rows
        .iter()
        .filter_map(|(from_id, ns, to_name, interwiki, _)| {
            if *ns == "0" && *interwiki == "''" {
                Some((*from_id, parser::string_value(to_name)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let from_names: Vec<Option<String>> = cache
        .mget(rows.iter().map(|(from_id, _)| *from_id).collect::<Vec<_>>())
        .expect("Tried `mget`ing the `from_names`");

    let to_ids: Vec<Option<String>> = cache
        .mget(
            rows.iter()
                .map(|(_, to_name)| cache::IdOf(to_name))
                .collect::<Vec<_>>(),
        )
        .expect("Tried `mget`ing the `to_ids`");

    // let mut row_pairs = Vec::with_capacity(rows.len());
    let mut lookup_pairs = Vec::with_capacity(rows.len());
    let mut pipe = Pipeline::with_capacity(rows.len());

    for ((from_id, _), (from_name, to_id)) in rows.iter().zip(from_names.iter().zip(to_ids.iter()))
    {
        let from_name = match from_name {
            Some(name) => name,
            None => continue,
        };
        let to_id = match to_id {
            Some(id) => id,
            None => continue,
        };

        pipe.sadd(cache::Aliases(to_id), from_id);
        // row_pairs.push((*from_id, from_name));
        lookup_pairs.push((cache::IdOf(from_name), to_id.as_str()));
    }

    pipe.query::<()>(cache)
        .expect("Tried `sadd`ing the aliases");

    // if !row_pairs.is_empty() {
    //     let _: () = cache
    //         .mset(&row_pairs)
    //         .expect("Tried `mset`ing the redirect lookups");
    // }
    if !lookup_pairs.is_empty() {
        let _: () = cache
            .mset(&lookup_pairs)
            .expect("Tried `mset`ing the redirect rows");
    }
}

pub fn pagelinks_line(cache: &mut redis::Connection, line: String) {
    let (_, rows) = parser::rows(parser::pagelink_row)(&line).unwrap();

    let rows = rows
        .iter()
        .filter_map(|(from_id, ns, to_name, from_ns)| {
            if *ns == "0" && *from_ns == "0" {
                Some((*from_id, parser::string_value(to_name)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return;
    }

    let to_ids: Vec<Option<String>> = cache
        .mget(
            rows.iter()
                .map(|(_, to_name)| cache::IdOf(to_name))
                .collect::<Vec<_>>(),
        )
        .expect("Tried `mget`ing the `to_ids`");

    let mut pipe = redis::Pipeline::with_capacity(rows.len());

    for ((from_id, _), to_id) in rows.iter().zip(to_ids.iter()) {
        let to_id = match to_id {
            Some(id) => id,
            None => continue,
        };

        pipe.sadd(cache::Links(from_id), to_id);
        // from_id.parse::<u32>().unwrap();
    }
    pipe.query::<()>(cache)
        .expect("Tried `sadd`ing the pagelinks in a pipeline");
}
