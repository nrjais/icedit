/// Text processing utilities for the editor

/// Check if a character should be considered a word boundary
/// This provides more intuitive word navigation behavior
pub fn is_word_boundary(ch: char) -> bool {
    // Whitespace is always a word boundary
    if ch.is_whitespace() {
        return true;
    }

    // Consider only specific punctuation as word boundaries
    match ch {
        // Common delimiters that should break words
        '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}'
        | '<' | '>' | '|' | '\\' | '/' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '+' | '='
        | '~' | '`' => true,
        // Keep underscores and hyphens as part of words for better programming/text experience
        '_' | '-' => false,
        // Other ASCII punctuation breaks words
        _ if ch.is_ascii_punctuation() => true,
        // Everything else (letters, numbers, unicode) is part of a word
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_boundaries() {
        // Whitespace should be boundaries
        assert!(is_word_boundary(' '));
        assert!(is_word_boundary('\t'));
        assert!(is_word_boundary('\n'));

        // Letters and numbers should not be boundaries
        assert!(!is_word_boundary('a'));
        assert!(!is_word_boundary('Z'));
        assert!(!is_word_boundary('5'));

        // Underscores and hyphens should not be boundaries (for programming/compound words)
        assert!(!is_word_boundary('_'));
        assert!(!is_word_boundary('-'));

        // Common punctuation should be boundaries
        assert!(is_word_boundary('.'));
        assert!(is_word_boundary(','));
        assert!(is_word_boundary('('));
        assert!(is_word_boundary(')'));
        assert!(is_word_boundary('['));
        assert!(is_word_boundary(']'));
        assert!(is_word_boundary('{'));
        assert!(is_word_boundary('}'));
        assert!(is_word_boundary('/'));
        assert!(is_word_boundary('\\'));
    }

    #[test]
    fn test_programming_scenarios() {
        // Test cases that should NOT break words (common in programming)
        assert!(!is_word_boundary('_')); // snake_case
        assert!(!is_word_boundary('-')); // kebab-case

        // Test cases that SHOULD break words
        assert!(is_word_boundary('.')); // method calls: obj.method
        assert!(is_word_boundary('(')); // function calls: func()
        assert!(is_word_boundary('[')); // array access: arr[0]
        assert!(is_word_boundary('{')); // object literals: {key: value}
        assert!(is_word_boundary(':')); // object literals, type annotations
        assert!(is_word_boundary(';')); // statement terminators
        assert!(is_word_boundary(',')); // parameter separators
    }

    #[test]
    fn test_text_scenarios() {
        // Test cases for natural language text
        assert!(is_word_boundary('.')); // sentence endings
        assert!(is_word_boundary(',')); // clause separators
        assert!(is_word_boundary('!')); // exclamations
        assert!(is_word_boundary('?')); // questions
        assert!(is_word_boundary('"')); // quotes
        assert!(is_word_boundary('\'')); // apostrophes in quotes

        // Hyphens in compound words should NOT break
        assert!(!is_word_boundary('-')); // well-known, twenty-one
    }
}
