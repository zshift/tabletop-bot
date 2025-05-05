use core::fmt::Display;

use log::error;

pub trait MapOrExit<V> {
    fn map_or_exit<M: AsRef<str>>(self, message: M) -> V;
}

impl<V, E: Display> MapOrExit<V> for Result<V, E> {
    fn map_or_exit<M: AsRef<str>>(self, message: M) -> V {
        match self {
            Ok(v) => v,
            Err(e) => {
                error!("{}: {}", message.as_ref(), e);
                std::process::exit(1);
            }
        }
    }
}
