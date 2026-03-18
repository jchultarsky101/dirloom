use std::sync::{Arc, Mutex};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use std::io;

/// Maximum number of log lines to keep in buffer
const MAX_LOG_LINES: usize = 100;

/// Thread-safe log buffer
#[derive(Debug, Clone, Default)]
pub struct LogBuffer {
    lines: Arc<Mutex<Vec<String>>>,
}

impl LogBuffer {
    /// Create a new LogBuffer
    pub fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::with_capacity(MAX_LOG_LINES))),
        }
    }

    /// Add a log line to the buffer
    pub fn add_line(&self, line: String) {
        let mut lines = self.lines.lock().unwrap();
        lines.push(line);
        if lines.len() > MAX_LOG_LINES {
            lines.remove(0);
        }
    }

    /// Get the last N log lines
    pub fn get_last(&self, n: usize) -> Vec<String> {
        let lines = self.lines.lock().unwrap();
        let start = lines.len().saturating_sub(n);
        lines[start..].to_vec()
    }

    /// Clear the log buffer
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut lines = self.lines.lock().unwrap();
        lines.clear();
    }
}

/// MakeWriter implementation that writes to the log buffer
#[derive(Clone)]
struct BufferWriter {
    buffer: LogBuffer,
}

impl io::Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(line) = String::from_utf8(buf.to_vec()) {
            let mut cleaned = line.trim().to_string();
            // Strip "INFO ", "DEBUG ", "WARN ", "ERROR " prefixes since we add our own emojis
            for prefix in ["INFO ", "DEBUG ", "WARN ", "ERROR ", "TRACE "] {
                if cleaned.starts_with(prefix) {
                    cleaned = cleaned[prefix.len()..].to_string();
                    break;
                }
            }
            if !cleaned.is_empty() {
                self.buffer.add_line(cleaned);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::writer::MakeWriter<'a> for BufferWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Initialize tracing with TUI-compatible logging
pub fn init_tui_tracing(buffer: LogBuffer) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let writer = BufferWriter {
        buffer: buffer.clone(),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(move || writer.clone())
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .without_time()
                .with_ansi(false),
        )
        .init();
}

/// Initialize standard tracing (for non-TUI mode)
#[allow(dead_code)]
pub fn init_standard_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info").add_directive("dirloom=debug".parse().unwrap()));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .pretty()
                .with_ansi(true),
        )
        .init();
}
