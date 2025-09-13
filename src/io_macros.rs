/// Waits for the result of an io function
#[cfg(not(feature = "async_io"))]
macro_rules! await_io {
    ($expr:expr) => { $expr }
}

/// Waits for the result of an io function
#[cfg(feature = "async_io")]
macro_rules! await_io {
    ($expr:expr) => { $expr.await }
}

pub(crate) use {
    await_io,
    cscsca_macros::io_fn,
};

#[cfg(test)]
pub(crate) use cscsca_macros::io_test;
