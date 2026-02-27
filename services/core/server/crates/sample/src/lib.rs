#![allow(clippy::needless_return, unexpected_cfgs)]

/// Adds two numbers together.
///
/// # Examples
///
/// ```rust
/// let result = sample::add(1, 2);
/// assert_eq!(result, 3);
/// ```
#[cfg_attr(flux, flux::sig(fn(left: i64, right: i64) -> i64{v: v == left + right}))]
pub fn add(left: i64, right: i64) -> i64 {
    return left + right;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }

    #[rstest]
    #[case(1, 2, 3)]
    #[case(0, 0, 0)]
    #[case(-1, -1, -2)]
    fn test_add_cases(#[case] left: i64, #[case] right: i64, #[case] expected: i64) {
        assert_eq!(add(left, right), expected);
    }

    proptest! {
        #[test]
        fn test_add_proptest(left in -1000i64..1000, right in -1000i64..1000) {
            let expected = left + right;
            assert_eq!(add(left, right), expected);
        }
    }
}

#[cfg(kani)]
mod verification {
    use kani::proof;

    use super::*;

    #[proof]
    fn proof_add() {
        let left = kani::any::<i64>();
        let right = kani::any::<i64>();
        let result = add(left, right);
        assert_eq!(result, left + right);
    }

    #[proof]
    fn proof_add_commutative() {
        let left = kani::any::<i64>();
        let right = kani::any::<i64>();
        let result1 = add(left, right);
        let result2 = add(right, left);
        assert_eq!(result1, result2);
    }
}

#[cfg(feature = "fuzz")]
pub mod fuzz {
    use afl::fuzz;

    use super::*;

    pub fn fuzz_add() {
        fuzz!(|data: (i64, i64)| {
            dbg!(data);
            let (left, right) = data;
            let expected = left + right;
            assert_eq!(add(left, right), expected);
        });
    }
}
