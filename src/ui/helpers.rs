/// Truncate a string to at most `max_chars` characters, appending "..." if truncated.
pub fn truncate_with_ellipsis(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}

/// Split a string into chunks of at most `chunk_size` characters.
pub fn char_chunks(s: &str, chunk_size: usize) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut start = 0;
    for (count, (i, _)) in s.char_indices().enumerate() {
        if count > 0 && count % chunk_size == 0 {
            chunks.push(&s[start..i]);
            start = i;
        }
    }
    if start < s.len() {
        chunks.push(&s[start..]);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate_with_ellipsis("abcdefghij", 5), "abcde...");
    }

    #[test]
    fn test_truncate_multibyte() {
        assert_eq!(truncate_with_ellipsis("café!", 4), "café...");
    }

    #[test]
    fn test_char_chunks_basic() {
        let chunks = char_chunks("abcdefgh", 3);
        assert_eq!(chunks, vec!["abc", "def", "gh"]);
    }

    #[test]
    fn test_char_chunks_multibyte() {
        let chunks = char_chunks("aécàè", 2);
        assert_eq!(chunks, vec!["aé", "cà", "è"]);
    }

    #[test]
    fn test_char_chunks_empty() {
        let chunks = char_chunks("", 5);
        assert_eq!(chunks, Vec::<&str>::new());
    }
}
