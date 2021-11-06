//! The strings module handles common string operations on strings from the command-line

/// Wraps a character iterator over a string,
/// and unescapes escaped ASCII control sequences as it comes across them.
///
/// A simple example of this would be rendering escaped newline and tab chars:
///
/// ```rust
/// use coreutils_core::strings::StringEscapeDecoder;
/// let s = StringEscapeDecoder::from("a\\nb\\tc");
/// assert_eq!("a\nb\tc", s.collect::<String>());
/// ```
///
/// See: https://en.wikipedia.org/wiki/Escape_sequences_in_C#Table_of_escape_sequences
pub struct StringEscapeDecoder<'a> {
    data: std::iter::Peekable<std::str::Chars<'a>>,
    min_size: usize,
    max_size: usize,
}

/// FIXME: It should be possible to convert this from anything that supports AsRef<str>
impl<'a> From<&'a str> for StringEscapeDecoder<'a> {
    fn from(buffer: &'a str) -> Self {
        let max_escapes = buffer.matches('\\').count();
        StringEscapeDecoder {
            data: buffer.chars().peekable(),
            min_size: buffer.len() - max_escapes,
            max_size: buffer.len(),
        }
    }
}

impl<'a> std::iter::Iterator for StringEscapeDecoder<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.data.next() {
            Some('\\') => {
                let (consume_next, char_to_emit) = match self.data.peek() {
                    Some('0') => (true, /* '\0' */ '\x00'),
                    Some('a') => (true, /* '\a' */ '\x07'),
                    Some('b') => (true, /* '\b' */ '\x08'),
                    Some('e') => (true, /* '\e' */ '\x1B'),
                    Some('f') => (true, /* '\f' */ '\x0C'),
                    Some('n') => (true, '\n'),
                    Some('r') => (true, '\r'),
                    Some('t') => (true, '\t'),
                    Some('v') => (true, /* '\v' */ '\x0B'),
                    Some('\\') => (true, '\\'),
                    Some('\'') => (true, '\''),
                    Some('"') => (true, '"'),
                    Some('?') => (true, /* '\?' */ '\x3F'),
                    // If the next character isn't a known ASCII escape code,
                    // or doesn't exist: Return the current '\\'
                    _ => (false, '\\'),
                };

                if consume_next {
                    self.data.next();
                    self.min_size -= 2;
                    self.max_size -= 2;
                } else {
                    self.min_size -= 1;
                    self.max_size -= 1;
                };
                Some(char_to_emit)
            },
            Some(c) => Some(c),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.min_size, Some(self.max_size))
    }
}

#[cfg(test)]
mod tests {
    use super::StringEscapeDecoder as SED;

    #[test]
    fn no_effect_on_empty_string() {
        let decoded = SED::from("").collect::<String>();
        assert_eq!("", decoded);
    }

    #[test]
    fn no_effect_when_there_are_no_escapes() {
        let decoded = SED::from("definitely no escapes to decode").collect::<String>();
        assert_eq!("definitely no escapes to decode", decoded);
    }

    #[test]
    fn decodes_ascii_byte_sequence() {
        let escaped_control_chars = vec![
            "\\0", "\\a", "\\b", "\\e", "\\f", "\\n", "\\r", "\\t", "\\v", "\\", "\'", "\\\"",
            "\\?",
        ];
        let expected_control_chars = vec![
            '\x00', '\x07', '\x08', '\x1B', '\x0C', '\n', '\r', '\t', '\x0B', '\\', '\'', '"', '?',
        ];
        for (idx, escaped_char) in escaped_control_chars.iter().enumerate() {
            let control_char = expected_control_chars[idx];
            let input = format!("delim{}seperated{}list", escaped_char, escaped_char);
            let expected = format!("delim{}seperated{}list", control_char, control_char);
            let output: String = SED::from(input.as_str()).collect();

            assert_eq!(expected, output, "Failed to decode: {}", input);
        }

        let decoded = SED::from("delim\\0seperated\\0list").collect::<String>();
        assert_eq!("delim\0seperated\0list", decoded);
    }
}
