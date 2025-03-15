use super::*;

pub(crate) struct Platform;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;
