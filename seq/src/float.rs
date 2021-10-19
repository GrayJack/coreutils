pub(crate) fn count_integer_digits(float: &str) -> usize {
    float.find('.').unwrap_or_else(|| float.len())
}


pub(crate) fn count_decimal_digits(float: &str) -> usize {
    let len = float.len();

    let decimal = float.find('.').map(|pos| pos + 1).unwrap_or(len);

    len - decimal
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_count_integer_digits() {
        assert_eq!(count_integer_digits("42"), 2);
        assert_eq!(count_integer_digits("12.22"), 2);
        assert_eq!(count_integer_digits("1.22"), 1);
        assert_eq!(count_integer_digits("1.3"), 1);
        assert_eq!(count_integer_digits("1"), 1);
        assert_eq!(count_integer_digits("-152.3"), 4); // Sign actually counts as a digit.
    }


    #[test]
    fn test_count_decimal_digits() {
        assert_eq!(count_decimal_digits("42"), 0);
        assert_eq!(count_decimal_digits("12.22"), 2);
        assert_eq!(count_decimal_digits("1.22"), 2);
        assert_eq!(count_decimal_digits("1.3"), 1);
        assert_eq!(count_decimal_digits("1"), 0);
        assert_eq!(count_decimal_digits("-152.3"), 1);
    }
}
