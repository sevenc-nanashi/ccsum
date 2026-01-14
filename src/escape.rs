pub fn escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\0' => escaped.push_str("\\0"),
            '\x07' => escaped.push_str("\\a"),
            '\x08' => escaped.push_str("\\b"),
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\x0b' => escaped.push_str("\\v"),
            '\x0c' => escaped.push_str("\\f"),
            '\r' => escaped.push_str("\\r"),
            '\x1b' => escaped.push_str("\\e"),
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(c),
        }
    }

    escaped
}

pub fn unescape(s: &str) -> anyhow::Result<String> {
    let mut unescaped = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('0') => unescaped.push('\0'),
                Some('a') => unescaped.push('\x07'),
                Some('b') => unescaped.push('\x08'),
                Some('t') => unescaped.push('\t'),
                Some('n') => unescaped.push('\n'),
                Some('v') => unescaped.push('\x0b'),
                Some('f') => unescaped.push('\x0c'),
                Some('r') => unescaped.push('\r'),
                Some('e') => unescaped.push('\x1b'),
                Some('\\') => unescaped.push('\\'),
                Some('\'') => unescaped.push('\''),
                Some('"') => unescaped.push('"'),
                Some(c) => return Err(anyhow::anyhow!("invalid escape sequence: \\{c}")),
                None => return Err(anyhow::anyhow!("incomplete escape sequence")),
            }
        } else {
            unescaped.push(c);
        }
    }

    Ok(unescaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(escape("hello, world!"), "hello, world!");
        assert_eq!(escape("hello, \"world\"!"), "hello, \\\"world\\\"!");
        assert_eq!(escape("hello, 'world'!"), "hello, \\'world\\'!");
        assert_eq!(escape("hello, \\world\\!"), "hello, \\\\world\\\\!");
        assert_eq!(escape("hello, \x07world\x08!"), "hello, \\aworld\\b!");
        assert_eq!(escape("hello, \tworld\n!"), "hello, \\tworld\\n!");
        assert_eq!(escape("hello, \x0bworld\x0c!"), "hello, \\vworld\\f!");
        assert_eq!(escape("hello, \rworld!"), "hello, \\rworld!");
        assert_eq!(escape("hello, \x1bworld!"), "hello, \\eworld!");
        assert_eq!(escape("hello, \0world!"), "hello, \\0world!");
    }

    #[test]
    fn test_unescape() {
        assert_eq!(unescape("hello, world!").unwrap(), "hello, world!");
        assert_eq!(
            unescape("hello, \\\"world\\\"!").unwrap(),
            "hello, \"world\"!"
        );
        assert_eq!(unescape("hello, \\'world\\'!").unwrap(), "hello, 'world'!");
        assert_eq!(
            unescape("hello, \\\\world\\\\!").unwrap(),
            "hello, \\world\\!"
        );
        assert_eq!(
            unescape("hello, \\aworld\\b!").unwrap(),
            "hello, \x07world\x08!"
        );
        assert_eq!(
            unescape("hello, \\tworld\\n!").unwrap(),
            "hello, \tworld\n!"
        );
        assert_eq!(
            unescape("hello, \\vworld\\f!").unwrap(),
            "hello, \x0bworld\x0c!"
        );
        assert_eq!(unescape("hello, \\rworld!").unwrap(), "hello, \rworld!");
        assert_eq!(unescape("hello, \\eworld!").unwrap(), "hello, \x1bworld!");
        assert_eq!(unescape("hello, \\0world!").unwrap(), "hello, \0world!");
    }
}
