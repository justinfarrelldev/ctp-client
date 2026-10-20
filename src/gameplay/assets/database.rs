//! Shared original text-record scanning, never a local game-rule interpreter.

/// Quoted braces are values, not record delimiters.
#[derive(Debug, PartialEq, Eq)]
struct Token {
    /// Original unquoted value or delimiter spelling.
    word: String,
    /// True only for an unquoted syntactic brace.
    delimiter: bool,
}

/// Finds an exact record field, not an incidental token in a comment.
pub(super) fn field<'data>(words: &'data [String], name: &str) -> Option<&'data str> {
    words
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

/// Reads balanced top-level database records while ignoring comments/quoted braces.
pub(super) fn records(text: &str) -> Vec<(String, Vec<String>)> {
    let tokens = tokenize(text);
    let mut records = Vec::new();
    let mut index = 0_usize;
    while index.saturating_add(1_usize) < tokens.len() {
        let next = &tokens[index.saturating_add(1_usize)];
        if !next.delimiter || next.word != "{" {
            index = index.saturating_add(1_usize);
            continue;
        }
        let name = tokens[index].word.clone();
        index = index.saturating_add(2_usize);
        let start = index;
        let mut depth = 1_u32;
        while index < tokens.len() && depth > 0_u32 {
            let token = &tokens[index];
            match (token.word.as_str(), token.delimiter) {
                ("{", true) => depth = depth.saturating_add(1_u32),
                ("}", true) => depth = depth.saturating_sub(1_u32),
                _ => {}
            }
            index = index.saturating_add(1_usize);
        }
        if depth == 0_u32 {
            records.push((
                name,
                tokens[start..index.saturating_sub(1_usize)]
                    .iter()
                    .map(|token| token.word.clone())
                    .collect(),
            ));
        }
    }
    records
}

/// Minimal game-database lexer preserving strings and dropping line comments.
fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut comment = false;
    for character in text.chars() {
        if comment {
            if character == '\n' {
                comment = false;
            } else {
                continue;
            }
        }
        if character == '"' {
            quoted = !quoted;
            if !quoted {
                tokens.push(Token {
                    word: core::mem::take(&mut word),
                    delimiter: false,
                });
            }
            continue;
        }
        if !quoted && character == '#' {
            comment = true;
            continue;
        }
        if !quoted && (character.is_whitespace() || matches!(character, '{' | '}')) {
            if !word.is_empty() {
                tokens.push(Token {
                    word: core::mem::take(&mut word),
                    delimiter: false,
                });
            }
            if matches!(character, '{' | '}') {
                tokens.push(Token {
                    word: character.to_string(),
                    delimiter: true,
                });
            }
        } else {
            word.push(character);
        }
    }
    if !word.is_empty() {
        tokens.push(Token {
            word,
            delimiter: false,
        });
    }
    tokens
}

/// Pure lexer regressions, with no installed or temporary files.
#[cfg(test)]
mod tests {
    use super::{field, records};

    #[test]
    fn quoted_braces_do_not_change_record_depth_or_hide_following_records() {
        let data = records(r#"A { Payload "{" Icon "one" } B { Payload "}" Icon "two" }"#);
        assert_eq!(
            data.len(),
            2_usize,
            "literal braces never consume adjacent records"
        );
        assert_eq!(
            field(&data[0_usize].1, "Payload"),
            Some("{"),
            "literal value preserved"
        );
        assert_eq!(
            field(&data[1_usize].1, "Icon"),
            Some("two"),
            "next record independently parsed"
        );
    }
}
