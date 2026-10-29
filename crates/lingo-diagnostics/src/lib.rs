use lingo_source::{SourceFile, Span};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub note: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span,
            note: None,
        }
    }

    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            span,
            note: None,
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn render(&self, file: &SourceFile) -> String {
        let (line_no, col_no) = file.location(self.span.start);
        let prefix = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };

        let mut out = format!("{prefix}: {}\n --> {}:{}:{}\n", self.message, file.name, line_no, col_no);

        if let Some(line) = file.get_line(line_no) {
            let line_num_str = format!("{line_no} | ");
            let indent = " ".repeat(line_num_str.len() + col_no.saturating_sub(1));
            let width = (self.span.end.saturating_sub(self.span.start)).max(1);
            let underline = "^".repeat(width);

            out.push_str(&format!("{line_num_str}{line}\n"));
            out.push_str(&format!("{indent}{underline}\n"));
        }

        if let Some(ref note) = self.note {
            out.push_str(&format!("  = note: {note}\n"));
        }

        out
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {} at {:?}", self.severity, self.message, self.span)
    }
}

impl std::error::Error for Diagnostic {}
