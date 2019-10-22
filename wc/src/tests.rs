use super::*;

struct TestReader<'a> {
    buf: &'a str,
    i:   usize,
}

impl<'a> TestReader<'a> {
    fn new(s: &'a str) -> Self { TestReader { buf: s, i: 0 } }
}

impl Read for TestReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let i = self.i;
        let n = out.len().min(self.buf.len() - i);
        let buf_ptr = self.buf.as_ptr();
        let out_ptr = out.as_mut_ptr();
        unsafe {
            buf_ptr.copy_to(out_ptr, n);
        }
        self.i += n;
        Ok(n)
    }
}

#[test]
fn wc_stdin() {
    let test_str = TestReader::new("This is a test string");
    let flags = WcFlags {
        print_bytes: true,
        print_chars: true,
        print_lines: true,
        print_words: true,
        print_max_line_len: true,
        pretty: false,
    };
    let res = get_formatted_result("-", &wc(test_str).unwrap(), flags);
    assert_eq!(res, String::from("1 5 22 22 21 "));
}

#[test]
fn wc_pretty_print() {
    let test_str = TestReader::new("This is a test string");
    let flags = WcFlags {
        print_bytes: true,
        print_chars: false,
        print_lines: true,
        print_words: false,
        print_max_line_len: false,
        pretty: true,
    };
    let res = get_formatted_result("test", &wc(test_str).unwrap(), flags);
    assert_eq!(
        res,
        String::from(
            "test
  lines: 1
  bytes: 22"
        )
    );
}
