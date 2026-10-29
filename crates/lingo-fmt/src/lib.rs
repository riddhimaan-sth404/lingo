pub struct Formatter;

impl Formatter {
    pub fn format_source(src: &str) -> String {
        let mut out = String::new();
        let mut indent_level: usize = 0;

        for line in src.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                out.push('\n');
                continue;
            }

            if trimmed.starts_with('#') {
                out.push_str(&"    ".repeat(indent_level));
                out.push_str(trimmed);
                out.push('\n');
                continue;
            }

            // Adjust indentation for unindent keywords (elif, else)
            let is_elif_or_else = trimmed.starts_with("elif") || trimmed.starts_with("else:");
            let effective_indent = if is_elif_or_else && indent_level > 0 {
                indent_level - 1
            } else {
                indent_level
            };

            out.push_str(&"    ".repeat(effective_indent));
            out.push_str(trimmed);
            out.push('\n');

            if trimmed.ends_with(':') {
                indent_level += 1;
            } else if trimmed.starts_with("return") && indent_level > 1 {
                // Return statement in nested branch
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_indentation() {
        let unformatted = "def main():\n  x=10\n  if x>5:\n    println(x)\n";
        let formatted = Formatter::format_source(unformatted);
        assert!(formatted.contains("    x=10"));
        assert!(formatted.contains("        println(x)"));
    }
}
