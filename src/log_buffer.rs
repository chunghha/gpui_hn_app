use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// Thread-safe log buffer with fixed capacity
#[derive(Clone)]
pub struct LogBuffer {
    lines: Arc<Mutex<VecDeque<String>>>,
    capacity: usize,
}

impl LogBuffer {
    /// Create a new log buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            lines: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    /// Append a log line to the buffer
    pub fn append(&self, line: String) {
        if let Ok(mut lines) = self.lines.lock() {
            if lines.len() >= self.capacity {
                lines.pop_front();
            }
            lines.push_back(line);
        }
    }

    /// Get all current log lines
    pub fn get_lines(&self) -> Vec<String> {
        self.lines
            .lock()
            .map(|lines| lines.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Clear all log lines
    #[allow(dead_code)]
    pub fn clear(&self) {
        if let Ok(mut lines) = self.lines.lock() {
            lines.clear();
        }
    }
}

/// A tracing layer that captures logs into a LogBuffer
pub struct LogBufferLayer {
    buffer: LogBuffer,
}

impl LogBufferLayer {
    pub fn new(buffer: LogBuffer) -> Self {
        Self { buffer }
    }
}

impl<S> Layer<S> for LogBufferLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = metadata.level();
        let target = metadata.target();

        // Format: [LEVEL] target: message
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let formatted = format!("[{}] {}: {}", level, target, visitor.message);
        self.buffer.append(formatted);
    }
}

/// Visitor to extract the message from a tracing event
#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Remove surrounding quotes if present
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer_creation() {
        let buffer = LogBuffer::new(100);
        assert_eq!(buffer.get_lines().len(), 0);
    }

    #[test]
    fn test_log_buffer_append() {
        let buffer = LogBuffer::new(100);
        buffer.append("Test log 1".to_string());
        buffer.append("Test log 2".to_string());

        let lines = buffer.get_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Test log 1");
        assert_eq!(lines[1], "Test log 2");
    }

    #[test]
    fn test_log_buffer_capacity() {
        let buffer = LogBuffer::new(3);
        buffer.append("Log 1".to_string());
        buffer.append("Log 2".to_string());
        buffer.append("Log 3".to_string());
        buffer.append("Log 4".to_string());

        let lines = buffer.get_lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Log 2");
        assert_eq!(lines[1], "Log 3");
        assert_eq!(lines[2], "Log 4");
    }

    #[test]
    fn test_log_buffer_clear() {
        let buffer = LogBuffer::new(100);
        buffer.append("Test log 1".to_string());
        buffer.append("Test log 2".to_string());

        buffer.clear();

        let lines = buffer.get_lines();
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_log_buffer_clone() {
        let buffer1 = LogBuffer::new(100);
        buffer1.append("Test log".to_string());

        let buffer2 = buffer1.clone();
        let lines = buffer2.get_lines();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Test log");
    }
}
