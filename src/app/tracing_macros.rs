#[cfg(debug_assertions)]
macro_rules! trace_hot_path {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! trace_hot_path {
    ($($arg:tt)*) => {};
}

#[cfg(debug_assertions)]
macro_rules! debug_hot_path {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_hot_path {
    ($($arg:tt)*) => {};
}

pub(crate) use trace_hot_path;
pub(crate) use debug_hot_path;