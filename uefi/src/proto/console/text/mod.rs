//! Text I/O.

mod input;
pub use self::input::{Input, Key, ScanCode};

mod output;
pub use self::output::{Color, Output, OutputMode};

#[cfg(feature = "platform")]
pub use output::OutputData;
