use super::*;

// Macro to assert that an expression matches a pattern.
macro_rules! assert_matches {
    ($xpr:expr, $pat:pat) => {
        match $xpr {
            $pat => true,
            ref xpr => panic!("assert_matches: '{:?}' doesn't match '{}'", xpr, stringify!($pat)),
        }
    };
}

#[test]
fn range_from_string() {
    assert_eq!(Range::from_string("2"), Ok(Range(1, 2)));
    assert_eq!(Range::from_string("-2"), Ok(Range(usize::min_value(), 2)));
    assert_eq!(Range::from_string("2-"), Ok(Range(1, usize::max_value())));
    assert_eq!(Range::from_string("2-5"), Ok(Range(1, 5)));

    assert_matches!(Range::from_string(""), Err(Error(_, _)));
    assert_matches!(Range::from_string("5-2"), Err(Error(_, _)));
    assert_matches!(Range::from_string("foo"), Err(Error(_, _)));
    assert_matches!(Range::from_string("2-0x12"), Err(Error(_, _)));
    assert_matches!(Range::from_string("-"), Err(Error(_, _)));
}

#[test]
fn rangeset_from_string() {
    assert_eq!(RangeSet::from_string("2"), Ok(RangeSet { points: vec![Range(1, 2)] }));
    assert_eq!(
        RangeSet::from_string("-2"),
        Ok(RangeSet { points: vec![Range(usize::min_value(), 2)] })
    );
    assert_eq!(RangeSet::from_string("2,3"), Ok(RangeSet { points: vec![Range(1, 3)] }));
    assert_eq!(RangeSet::from_string("2-3"), Ok(RangeSet { points: vec![Range(1, 3)] }));
    assert_eq!(RangeSet::from_string("2-3,3-5,4-6"), Ok(RangeSet { points: vec![Range(1, 6)] }));
    assert_eq!(RangeSet::from_string("4-6,3-5,2-3"), Ok(RangeSet { points: vec![Range(1, 6)] }));
    assert_eq!(
        RangeSet::from_string("2,5-10"),
        Ok(RangeSet { points: vec![Range(1, 2), Range(4, 10)] })
    );
    assert_eq!(
        RangeSet::from_string("2,5-"),
        RangeSet::from_vec(vec![Range(1, 2), Range(4, usize::max_value())])
    );
    assert_eq!(
        RangeSet::from_string("-2,5-"),
        Ok(RangeSet { points: vec![Range(usize::min_value(), 2), Range(4, usize::max_value())] })
    );
}

fn complement_rangeset_helper(ranges: Vec<Range>, expected: Vec<Range>) {
    let mut range_set = RangeSet::from_vec(ranges).unwrap();
    range_set.complement();
    assert_eq!(range_set, RangeSet::from_vec(expected).unwrap());
}

#[test]
fn completment_rangeset() {
    complement_rangeset_helper(vec![Range(usize::min_value(), usize::max_value())], vec![]);
    complement_rangeset_helper(vec![Range(usize::min_value(), 5)], vec![Range(
        5,
        usize::max_value(),
    )]);
    complement_rangeset_helper(vec![Range(5, usize::max_value())], vec![Range(
        usize::min_value(),
        5,
    )]);
    complement_rangeset_helper(vec![Range(1, 5)], vec![
        Range(usize::min_value(), 1),
        Range(5, usize::max_value()),
    ]);
    complement_rangeset_helper(vec![Range(1, 5), Range(8, 12)], vec![
        Range(usize::min_value(), 1),
        Range(5, 8),
        Range(12, usize::max_value()),
    ]);
    complement_rangeset_helper(vec![Range(usize::min_value(), 5), Range(8, 12)], vec![
        Range(5, 8),
        Range(12, usize::max_value()),
    ]);
    complement_rangeset_helper(vec![Range(5, 8), Range(12, usize::max_value())], vec![
        Range(0, 5),
        Range(8, 12),
    ]);
}
