use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub path: Option<PathBuf>,
    pub src: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(name: impl Into<String>, path: Option<PathBuf>, src: String) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }

        Self {
            name: name.into(),
            path,
            src,
            line_starts,
        }
    }

    pub fn location(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.src.len());
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };

        let col = offset - self.line_starts[line_idx] + 1;
        (line_idx + 1, col)
    }

    pub fn get_line(&self, line_num: usize) -> Option<&str> {
        if line_num == 0 || line_num > self.line_starts.len() {
            return None;
        }

        let start = self.line_starts[line_num - 1];
        let end = if line_num < self.line_starts.len() {
            self.line_starts[line_num].saturating_sub(1)
        } else {
            self.src.len()
        };

        let slice = &self.src[start..end];
        Some(slice.trim_end_matches('\r'))
    }
}

#[derive(Default, Debug, Clone)]
pub struct SourceManager {
    files: Vec<Arc<SourceFile>>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, name: impl Into<String>, path: Option<PathBuf>, src: String) -> Arc<SourceFile> {
        let file = Arc::new(SourceFile::new(name, path, src));
        self.files.push(file.clone());
        file
    }

    pub fn load_file(&mut self, path: &Path) -> std::io::Result<Arc<SourceFile>> {
        let src = std::fs::read_to_string(path)?;
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
        Ok(self.add_file(name, Some(path.to_path_buf()), src))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_locations() {
        let file = SourceFile::new("test.ln", None, "line 1\nline 2\nline 3".into());
        assert_eq!(file.location(0), (1, 1));
        assert_eq!(file.location(7), (2, 1));
        assert_eq!(file.get_line(2), Some("line 2"));
    }
}
