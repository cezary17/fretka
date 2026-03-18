use super::Truncator;

pub struct MaxLengthTruncator {
    pub max_chars: usize,
}

impl MaxLengthTruncator {
    pub fn new(max_chars: usize) -> Self {
        Self { max_chars }
    }
}

impl Truncator for MaxLengthTruncator {
    fn truncate(&self, text: &str) -> String {
        if text.len() <= self.max_chars {
            return text.to_string();
        }
        // Find a char boundary at or before max_chars
        let mut end = self.max_chars;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}\n[truncated]", &text[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_unchanged() {
        let t = MaxLengthTruncator::new(100);
        assert_eq!(t.truncate("hello"), "hello");
    }

    #[test]
    fn exact_length_unchanged() {
        let t = MaxLengthTruncator::new(5);
        assert_eq!(t.truncate("hello"), "hello");
    }

    #[test]
    fn long_text_truncated() {
        let t = MaxLengthTruncator::new(5);
        let result = t.truncate("hello world");
        assert_eq!(result, "hello\n[truncated]");
    }

    #[test]
    fn respects_char_boundaries() {
        let t = MaxLengthTruncator::new(4);
        // "café" — 'é' is 2 bytes, so byte index 4 is mid-char
        let result = t.truncate("café rest");
        assert!(result.ends_with("[truncated]"));
        // Should truncate before the 'é' since byte 4 is mid-char
        assert!(result.starts_with("caf"));
    }

    #[test]
    fn empty_text_unchanged() {
        let t = MaxLengthTruncator::new(100);
        assert_eq!(t.truncate(""), "");
    }

    #[test]
    fn single_char_limit() {
        let t = MaxLengthTruncator::new(1);
        assert_eq!(t.truncate("hello"), "h\n[truncated]");
    }

    #[test]
    fn multibyte_only_text() {
        let t = MaxLengthTruncator::new(6);
        // Each emoji is 4 bytes: "🦀🦀" = 8 bytes
        let result = t.truncate("🦀🦀");
        // Can only fit one emoji (4 bytes), second starts at byte 4
        assert_eq!(result, "🦀\n[truncated]");
    }

    #[test]
    fn truncated_output_contains_marker() {
        let t = MaxLengthTruncator::new(3);
        let result = t.truncate("abcdef");
        assert!(result.contains("[truncated]"));
        assert!(result.starts_with("abc"));
    }

    #[test]
    fn preserves_newlines_within_limit() {
        let t = MaxLengthTruncator::new(20);
        assert_eq!(t.truncate("line1\nline2"), "line1\nline2");
    }
}
