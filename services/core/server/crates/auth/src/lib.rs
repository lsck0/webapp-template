#![allow(clippy::needless_return)]

/// Adds two numbers together.
///
/// # Examples
///
/// ```rust
/// # use sample::add;
///
/// let result = add(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn add(left: i64, right: i64) -> i64 {
    return left + right;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(add(1, 2), 3);
    }
}
