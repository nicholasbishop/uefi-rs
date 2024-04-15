//! This optional feature adds support for the `log` crate, providing
//! a custom logger implementation which writes to a UEFI text output protocol.
//!
//! The main export of this module is the `Logger` structure,
//! which implements the `log` crate's trait `Log`.
//!
//! # Implementation details
//!
//! The implementation is not the most efficient, since there is no buffering done,
//! and the messages have to be converted from UTF-8 to UEFI's UCS-2.
//!
//! The last part also means that some Unicode characters might not be
//! supported by the UEFI console. Don't expect emoji output support.

use crate::proto::console::text::Output;
use crate::table::boot::BootServices;

use core::fmt::{self, Write};
use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

static TSC_FREQ: AtomicU64 = AtomicU64::new(0);

/// Logging implementation which writes to a UEFI output stream.
///
/// If this logger is used as a global logger, you must disable it using the
/// `disable` method before exiting UEFI boot services in order to prevent
/// undefined behaviour from inadvertent logging.
#[derive(Debug)]
pub struct Logger {
    writer: AtomicPtr<Output>,
}

impl Logger {
    /// Creates a new logger.
    ///
    /// The logger is initially disabled. Call [`set_output`] to enable it.
    ///
    /// [`set_output`]: Self::set_output
    #[must_use]
    pub const fn new() -> Self {
        Logger {
            writer: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Get the output pointer (may be null).
    #[must_use]
    fn output(&self) -> *mut Output {
        self.writer.load(Ordering::Acquire)
    }

    /// Set the [`Output`] to which the logger will write.
    ///
    /// If a null pointer is passed for `output`, this method is equivalent to
    /// calling [`disable`].
    ///
    /// # Safety
    ///
    /// The `output` pointer must either be null or point to a valid [`Output`]
    /// object. That object must remain valid until the logger is either
    /// disabled, or `set_output` is called with a different `output`.
    ///
    /// You must arrange for the [`disable`] method to be called or for this
    /// logger to be otherwise discarded before boot services are exited.
    ///
    /// [`disable`]: Self::disable
    pub unsafe fn set_output(&self, output: *mut Output) {
        self.writer.store(output, Ordering::Release);
    }

    /// Disable the logger.
    pub fn disable(&self) {
        unsafe { self.set_output(ptr::null_mut()) }
    }

    /// TODO
    pub fn init_freq(boot_services: &BootServices) {
        let t1 = get_timestamp();
        boot_services.stall(1_000_000);
        let t2 = get_timestamp();
        let d = t2 - t1;
        TSC_FREQ.store(d, Ordering::Release);
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        !self.output().is_null()
    }

    fn log(&self, record: &log::Record) {
        let output = self.output();

        if !output.is_null() {
            let writer = unsafe { &mut *output };
            let result = DecoratedLog::write(
                writer,
                record.level(),
                record.args(),
                record.file().unwrap_or("<unknown file>"),
                record.line().unwrap_or(0),
            );

            // Some UEFI implementations, such as the one used by VirtualBox,
            // may intermittently drop out some text from SimpleTextOutput and
            // report an EFI_DEVICE_ERROR. This will be reported here as an
            // `fmt::Error`, and given how the `log` crate is designed, our main
            // choices when that happens are to ignore the error or panic.
            //
            // Ignoring errors is bad, especially when they represent loss of
            // precious early-boot system diagnosis data, so we panic by
            // default. But if you experience this problem and want your UEFI
            // application to keep running when it happens, you can disable the
            // `panic-on-logger-errors` cargo feature. If you do so, logging errors
            // will be ignored by `uefi-rs` instead.
            //
            if cfg!(feature = "panic-on-logger-errors") {
                result.unwrap()
            }
        }
    }

    fn flush(&self) {
        // This simple logger does not buffer output.
    }
}

// The logger is not thread-safe, but the UEFI boot environment only uses one processor.
unsafe impl Sync for Logger {}
unsafe impl Send for Logger {}

/// Writer wrapper which prints a log level in front of every line of text
///
/// This is less easy than it sounds because...
///
/// 1. The fmt::Arguments is a rather opaque type, the ~only thing you can do
///    with it is to hand it to an fmt::Write implementation.
/// 2. Without using memory allocation, the easy cop-out of writing everything
///    to a String then post-processing is not available.
///
/// Therefore, we need to inject ourselves in the middle of the fmt::Write
/// machinery and intercept the strings that it sends to the Writer.
struct DecoratedLog<'writer, 'a, W: fmt::Write> {
    writer: &'writer mut W,
    log_level: log::Level,
    at_line_start: bool,
    file: &'a str,
    line: u32,
}

impl<'writer, 'a, W: fmt::Write> DecoratedLog<'writer, 'a, W> {
    // Call this method to print a level-annotated log
    fn write(
        writer: &'writer mut W,
        log_level: log::Level,
        args: &fmt::Arguments,
        file: &'a str,
        line: u32,
    ) -> fmt::Result {
        let mut decorated_writer = Self {
            writer,
            log_level,
            at_line_start: true,
            file,
            line,
        };
        writeln!(decorated_writer, "{}", *args)
    }
}

fn get_timestamp() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

impl<'writer, 'a, W: fmt::Write> fmt::Write for DecoratedLog<'writer, 'a, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let timestamp = get_timestamp();
        let mut last_timestamp = LAST_TIMESTAMP.load(Ordering::Acquire);
        if last_timestamp == 0 {
            last_timestamp = timestamp;
        }
        let duration = timestamp - last_timestamp;
        LAST_TIMESTAMP.store(timestamp, Ordering::Release);

        let freq = TSC_FREQ.load(Ordering::Acquire);

        //let d = (duration as f64) / 100_000.0f64;
        let d = (duration as f64) / (freq as f64);

        // Split the input string into lines
        let mut lines = s.lines();

        // The beginning of the input string may actually fall in the middle of
        // a line of output. We only print the log level if it truly is at the
        // beginning of a line of output.
        let first = lines.next().unwrap_or("");
        if self.at_line_start {
            write!(
                self.writer,
                "[{:7.2}] [{:>5}]: {:>12}@{:03}: ",
                d, self.log_level, self.file, self.line
            )?;
            self.at_line_start = false;
        }
        write!(self.writer, "{first}")?;

        // For the remainder of the line iterator (if any), we know that we are
        // truly at the beginning of lines of output.
        for line in lines {
            let level = self.log_level;
            write!(self.writer, "\n{level}: {line}")?;
        }

        // If the string ends with a newline character, we must 1/propagate it
        // to the output (it was swallowed by the iteration) and 2/prepare to
        // write the log level of the beginning of the next line (if any).
        if let Some('\n') = s.chars().next_back() {
            writeln!(self.writer)?;
            self.at_line_start = true;
        }
        Ok(())
    }
}
