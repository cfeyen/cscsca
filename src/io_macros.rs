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

/// Creates a test that wraps async io
#[cfg(test)]
#[cfg(not(feature = "async_io"))]
macro_rules! io_test {
    (fn $name:ident() $block:block) => {
        #[test]
        fn $name() $block
    };
}

/// Creates a test that wraps async io
#[cfg(test)]
#[cfg(feature = "async_io")]
macro_rules! io_test {
    (fn $name:ident() $block:block) => {
        #[test]
        fn $name() {
            pollster::block_on(async $block);
        }
    };
}

pub(crate) use {
    await_io,
    cscsca_macros::io_fn,
};

#[cfg(test)]
pub(crate) use io_test;
