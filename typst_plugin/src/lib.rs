use wasm_minimal_protocol::{initiate_protocol, wasm_func};

use omni::node::Db;

initiate_protocol!();

/// `nodes_toml`: content of nodes.toml
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
fn parse_link(nodes_toml: &[u8], file_part: &[u8], heading_part: &[u8], alias: &[u8]) -> Vec<u8> {
    let db: Db = toml::from_slice(nodes_toml).unwrap();
    let file_splits: Vec<_> = file_part.split(|c| *c == b'.').collect();

    "".as_bytes().to_vec()
}

#[wasm_func]
fn hello() -> Vec<u8> {
    "hello".as_bytes().to_vec()
}
