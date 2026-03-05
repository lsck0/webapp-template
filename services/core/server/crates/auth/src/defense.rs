#![allow(clippy::needless_return)]

use std::time::{Duration, Instant};

use rand::RngExt;

/// Minimum time a credential-checking operation should take, to prevent
/// timing side-channels from leaking whether a user exists, which step
/// failed, etc.
const MIN_RESPONSE_TIME: Duration = Duration::from_millis(200);

/// Maximum random jitter added on top of the minimum response time.
const MAX_JITTER: Duration = Duration::from_millis(15);

/// Execute `f` and ensure the total wall-clock time is at least
/// `MIN_RESPONSE_TIME` plus a random jitter of 0–`MAX_JITTER`.
///
/// This prevents attackers from distinguishing between:
/// - "user not found" (fast return)
/// - "wrong password" (slower due to argon2)
/// - "correct password" (same argon2 cost)
///
/// The random jitter further obscures the actual processing time.
pub fn constant_time<T>(f: impl FnOnce() -> T) -> T {
    let jitter = Duration::from_micros(rand::rng().random_range(0..MAX_JITTER.as_micros() as u64));
    let target = MIN_RESPONSE_TIME + jitter;

    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();

    if elapsed < target {
        std::thread::sleep(target - elapsed);
    }

    return result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_pads_fast_operations() {
        let start = Instant::now();
        let result = constant_time(|| 42);
        let elapsed = start.elapsed();

        assert_eq!(result, 42);
        assert!(
            elapsed >= MIN_RESPONSE_TIME,
            "should take at least {:?}, took {:?}",
            MIN_RESPONSE_TIME,
            elapsed
        );
    }

    #[test]
    fn test_constant_time_does_not_delay_slow_operations() {
        let slow_duration = MIN_RESPONSE_TIME + MAX_JITTER + Duration::from_millis(50);

        let start = Instant::now();
        constant_time(|| std::thread::sleep(slow_duration));
        let elapsed = start.elapsed();

        // Should not add extra delay beyond the operation itself
        assert!(
            elapsed < slow_duration + Duration::from_millis(10),
            "should not add significant delay, took {:?}",
            elapsed
        );
    }

    #[test]
    fn test_constant_time_propagates_result() {
        let result = constant_time(|| String::from("hello"));
        assert_eq!(result, "hello");
    }
}
