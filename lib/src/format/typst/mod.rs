pub mod compile;
pub use compile::CompileError;
pub use compile::compile;

pub mod query;
pub use query::QueryError;
pub use query::QueryParams;
pub use query::query;

pub enum Format {
    Pdf,
    Html,
}

impl Default for Format {
    fn default() -> Self {
        Self::Pdf
    }
}
