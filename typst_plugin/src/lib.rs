// super fragile.
// be very careful on what you use here as ANYTHING that relies on IO will result in a wasm trap,
// and you will spend 8 hours debugging it.

use wasm_minimal_protocol::{initiate_protocol, wasm_func};

use omni::{config::Config, link::FilePart, node};

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
/// `target`: url to link to or "ghost"
/// `to`: id of target node or "ghost"
#[wasm_func]
fn parse_link(file_part: &[u8], _heading_part: &[u8], alias: &[u8]) -> Vec<u8> {
    let lock = STATE.lock();
    let state = lock.as_ref().unwrap();

    // return (toml::to_string_pretty(&state.config)
    //     .unwrap_or("NOTHING".to_string())
    //     .to_string()
    //     + ",ghost,ghost")
    //     .into_bytes();
    //
    // return format!("{},ghost,ghost", String::from_utf8_lossy(file_part)).into_bytes();

    // create a FilePart from the raw dot separated one
    let file_splits: Vec<_> = file_part.split(|c| *c == b'.').collect();
    let file_part = if file_splits.is_empty() {
        return b"err: empty file part".to_vec();
    } else if file_splits.len() == 1 {
        let title = String::from_utf8_lossy(file_splits[0]).to_string();

        FilePart::Name(title)
    } else {
        let mut path = vec![];
        for component in file_splits.iter().take(file_splits.len() - 1) {
            path.push(String::from_utf8_lossy(component).to_string());
        }

        let last = file_splits.last().expect("slice should never be empty");
        let title = String::from_utf8_lossy(last).to_string();
        FilePart::PathAndName(path, title)
    };

    let maybe_node = match state.db.find_from_filepart(&file_part, &state.config) {
        Ok(node) => Some(node),
        Err(node::Error::NameNotFound(_)) => None,
        Err(err) => return format!("err: {}", err).into_bytes(),
    };

    // TODO: create a HeadingPart from the raw dot separated one
    // let _heading_splits: Vec<_> = heading_part.split(|c| *c == b'.').collect();

    match maybe_node {
        Some(node) => {
            let content = if alias.is_empty() {
                "PLACEHOLDER..."
            } else {
                &String::from_utf8_lossy(alias)
            };

            format!("{},{},{}", content, node.path, node.id.0).into_bytes()
        }
        None => {
            let content = if alias.is_empty() {
                "PLACEHOLDER GHOST"
            } else {
                &String::from_utf8_lossy(alias)
            };

            format!("{},ghost,ghost", content).into_bytes()
        }
    }
}

#[wasm_func]
fn hello() -> Vec<u8> {
    "hello".as_bytes().to_vec()
}
