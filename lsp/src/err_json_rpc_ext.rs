use std::error::Error;

use tower_lsp_server::jsonrpc;

pub trait ResultToJsonRpcExt<T> {
    fn rpc(self) -> jsonrpc::Result<T>;
}

impl<T, E: Error> ResultToJsonRpcExt<T> for Result<T, E> {
    fn rpc(self) -> jsonrpc::Result<T> {
        self.map_err(|err| jsonrpc::Error {
            code: jsonrpc::ErrorCode::ServerError(-32803),
            message: err.to_string().into(),
            data: None,
        })
    }
}
