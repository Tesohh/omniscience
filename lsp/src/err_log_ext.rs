use std::fmt::Display;

pub trait ErrLogExt {
    fn log_err(self, msg: &str) -> Self;
}

impl<T, E: Display> ErrLogExt for Result<T, E> {
    fn log_err(self, msg: &str) -> Self {
        if let Err(err) = &self {
            tracing::error!("{}: {}", msg, err);
        };
        self
    }
}
