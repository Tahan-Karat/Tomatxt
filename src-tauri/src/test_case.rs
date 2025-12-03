#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_times() {
        assert_eq!(to_times(1), 60);
        assert_eq!(to_times(5), 300);
    }

    #[test]
    fn test_tick() {
        assert_eq!(tick(10), 9);
        assert_eq!(tick(1), 0);
        assert_eq!(tick(0), 0);
    }

    #[test]
    fn test_is_finished() {
        assert!(is_finished(0));
        assert!(!is_finished(10));
    }

    #[test]
    fn test_next_phase() {
        assert_eq!(next_phase(true), false);
        assert_eq!(next_phase(false), true);
    }
}

