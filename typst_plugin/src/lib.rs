use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_minimal_protocol::{initiate_protocol, wasm_func};

// use omni::{config::Config, node};
use omni::{config::Config, node};

initiate_protocol!();

struct State {
    pub db: node::Db,
    pub config: Config,
}

static STATE: spin::Mutex<Option<State>> = spin::Mutex::new(None);

#[wasm_func]
fn init(nodes_toml: &[u8], config_toml: &[u8]) -> Vec<u8> {
    let maybe_db = toml::from_slice::<node::Db>(nodes_toml);
    let db = match maybe_db {
        Ok(new_db) => new_db,
        Err(err) => return (format!("err: {}", err)).into_bytes(),
    };

    let maybe_config = toml::from_slice::<Config>(config_toml);
    let config = match maybe_config {
        Ok(new_config) => new_config,
        Err(err) => return (format!("err: {}", err)).into_bytes(),
    };

    let mut guard = STATE.lock();

    *guard = Some(State { db, config });

    b"ok".to_vec()
}

/// `file_part`: bytes
/// `heading_part`: bytes or ""
/// `alias`: bytes or ""
///
/// returns (comma separated bytes):
/// `content`: content to display as the link in rendered output
/// `target`: url to link to
/// `to`: id of target node or "ghost"
/// TODO: use plugin.transition
#[wasm_func]
fn parse_link(file_part: &[u8], heading_part: &[u8], alias: &[u8]) -> Vec<u8> {
    // let db = db().lock().unwrap().as_ref().unwrap();
    // let config = config().lock().unwrap().as_ref().unwrap();

    let file_splits: Vec<_> = file_part.split(|c| *c == b'.').collect();
    let _heading_splits: Vec<_> = heading_part.split(|c| *c == b'.').collect(); // TODO:

    // db.find_from_filepart(part, config);

    "".as_bytes().to_vec()
}

#[wasm_func]
fn hello() -> Vec<u8> {
    "hello".as_bytes().to_vec()
}
