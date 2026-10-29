use lingo_lexer::Lexer;
use lingo_parser::Parser;
use lingo_source::SourceFile;

#[derive(Debug, Clone, PartialEq)]
pub struct LspDiagnostic {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletionItem {
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HoverInfo {
    pub title: String,
    pub description: String,
}

pub struct LspEngine;

impl LspEngine {
    pub fn validate_document(name: &str, src: &str) -> Vec<LspDiagnostic> {
        let file = SourceFile::new(name, None, src.to_string());
        let mut lexer = Lexer::new(&file);
        let tokens = match lexer.tokenize() {
            Ok(toks) => toks,
            Err(err) => {
                let (line, col) = file.location(err.span.start);
                return vec![LspDiagnostic {
                    line,
                    col,
                    message: err.message,
                }];
            }
        };

        let mut parser = Parser::new(tokens);
        if let Err(err) = parser.parse_program() {
            let (line, col) = file.location(err.span.start);
            return vec![LspDiagnostic {
                line,
                col,
                message: err.message,
            }];
        }

        Vec::new()
    }

    pub fn get_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem { label: "def".into(), detail: "Function definition".into() },
            CompletionItem { label: "let".into(), detail: "Constrained variable declaration".into() },
            CompletionItem { label: "if".into(), detail: "Conditional statement".into() },
            CompletionItem { label: "while".into(), detail: "While loop".into() },
            CompletionItem { label: "println".into(), detail: "Print value with newline".into() },
            CompletionItem { label: "Channel".into(), detail: "Create concurrency channel".into() },
            CompletionItem { label: "file_read".into(), detail: "Read file contents to string".into() },
            CompletionItem { label: "file_write".into(), detail: "Write string to file".into() },
            CompletionItem { label: "math_sqrt".into(), detail: "Square root of number".into() },
        ]
    }

    pub fn get_hover(word: &str) -> Option<HoverInfo> {
        match word {
            "println" => Some(HoverInfo {
                title: "fn println(val: Any) -> void".into(),
                description: "Prints a value followed by a newline to standard output.".into(),
            }),
            "Channel" => Some(HoverInfo {
                title: "fn Channel() -> Channel".into(),
                description: "Constructs a thread-safe message passing channel.".into(),
            }),
            "let" => Some(HoverInfo {
                title: "keyword let".into(),
                description: "Declares a statically typed, constrained variable: `let x: i64 = 42`.".into(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_diagnostics() {
        let valid_src = "def main():\n    println(42)\n";
        let diags = LspEngine::validate_document("test.ln", valid_src);
        assert!(diags.is_empty());

        let invalid_src = "def main(\n";
        let diags2 = LspEngine::validate_document("test.ln", invalid_src);
        assert!(!diags2.is_empty());
    }
}
