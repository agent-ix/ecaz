use color_eyre::eyre::{Context, Result};
use indicatif::ProgressBar;
use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tracing_subscriber::fmt::writer::MakeWriter;

static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

fn log_file_slot() -> &'static Mutex<Option<File>> {
    LOG_FILE.get_or_init(|| Mutex::new(None))
}

pub fn init(path: Option<&Path>) -> Result<()> {
    let mut guard = log_file_slot()
        .lock()
        .expect("log file mutex should not be poisoned");
    *guard = match path {
        Some(path) => {
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)
                    .wrap_err_with(|| format!("creating {}", parent.display()))?;
            }
            Some(File::create(path).wrap_err_with(|| format!("creating {}", path.display()))?)
        }
        None => None,
    };
    Ok(())
}

pub fn log_file_enabled() -> bool {
    log_file_slot()
        .lock()
        .expect("log file mutex should not be poisoned")
        .is_some()
}

fn append_log_copy(buf: &[u8]) -> io::Result<()> {
    let mut guard = log_file_slot()
        .lock()
        .expect("log file mutex should not be poisoned");
    if let Some(file) = guard.as_mut() {
        file.write_all(buf)?;
        file.flush()?;
    }
    Ok(())
}

fn write_mirrored(mut stream: impl Write, rendered: &str) {
    let bytes = rendered.as_bytes();
    let _ = stream.write_all(bytes);
    let _ = stream.flush();
    let _ = append_log_copy(bytes);
}

pub fn stdout_ln(args: fmt::Arguments<'_>) {
    write_mirrored(io::stdout(), &format!("{args}\n"));
}

pub fn stderr_ln(args: fmt::Arguments<'_>) {
    write_mirrored(io::stderr(), &format!("{args}\n"));
}

pub fn progress_bar(len: u64) -> ProgressBar {
    if log_file_enabled() {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(len)
    }
}

pub struct StderrMirror;

impl<'a> MakeWriter<'a> for StderrMirror {
    type Writer = MirrorStderr;

    fn make_writer(&'a self) -> Self::Writer {
        MirrorStderr
    }
}

pub struct MirrorStderr;

impl Write for MirrorStderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut stderr = io::stderr();
        stderr.write_all(buf)?;
        stderr.flush()?;
        append_log_copy(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stderr().flush()?;
        let mut guard = log_file_slot()
            .lock()
            .expect("log file mutex should not be poisoned");
        if let Some(file) = guard.as_mut() {
            file.flush()?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! ecaz_println {
    () => {
        $crate::output::stdout_ln(format_args!(""))
    };
    ($($arg:tt)*) => {
        $crate::output::stdout_ln(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! ecaz_eprintln {
    () => {
        $crate::output::stderr_ln(format_args!(""))
    };
    ($($arg:tt)*) => {
        $crate::output::stderr_ln(format_args!($($arg)*))
    };
}
