//! The strings module handles common string operations on strings from the command-line

/// Wraps a character iterator over a string,
/// and unescapes escaped ASCII control sequences as it comes across them.
///
/// A simple example of this would be rendering escaped newline and tab chars:
///
/// ```rust
/// let s = StringEscapeDecoder::from(&"a\\nb\\tc")
/// assert_eq!("a\nb\tc", a.collect());
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
                    Some('0') => (true, /* '\0' */ 0x00 as char),
                    Some('a') => (true, /* '\a' */ 0x07 as char),
                    Some('b') => (true, /* '\b' */ 0x08 as char),
                    Some('e') => (true, /* '\e' */ 0x1B as char),
                    Some('f') => (true, /* '\f' */ 0x0C as char),
                    Some('n') => (true, '\n'),
                    Some('r') => (true, '\r'),
                    Some('t') => (true, '\t'),
                    Some('v') => (true, /* '\v' */ 0x0B as char),
                    Some('\\') => (true, '\\'),
                    Some('\'') => (true, '\''),
                    Some('"') => (true, '"'),
                    Some('?') => (true, /* '\?' */ 0x3F as char),
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

