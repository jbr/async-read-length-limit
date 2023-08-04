use async_read_length_limit::LengthLimitExt;
use futures_lite::{future::block_on, io::Cursor, AsyncReadExt};

const MAX_MEMORY_TO_ALLOCATE: usize = 1024 * 1024;
const ITERATIONS: usize = 100;

#[test]
pub fn under_limit() {
    for _ in 0..ITERATIONS {
        let limit = fastrand::usize(2..MAX_MEMORY_TO_ALLOCATE);
        let input_length = fastrand::usize(1..limit);
        let input: Vec<_> = std::iter::repeat_with(|| fastrand::u8(..))
            .take(input_length)
            .collect();
        let cursor = Cursor::new(&input);
        let mut output = Vec::new();
        let result = block_on(cursor.limit_bytes(limit).read_to_end(&mut output));
        assert_eq!(input_length, result.unwrap());
        assert_eq!(input_length, output.len());
        assert_eq!(input, output);
    }
}

#[test]
pub fn over_limit() {
    for _ in 0..1000 {
        let limit = fastrand::usize(1..MAX_MEMORY_TO_ALLOCATE - 1);
        let input_length = fastrand::usize(limit..MAX_MEMORY_TO_ALLOCATE);
        let input: Vec<_> = std::iter::repeat_with(|| fastrand::u8(..))
            .take(input_length)
            .collect();
        let cursor = Cursor::new(&input);
        let mut output = Vec::new();
        let result = block_on(cursor.limit_bytes(limit).read_to_end(&mut output));
        assert!(result.is_err());
        assert_eq!(output.len(), limit);
        assert_eq!(input[..limit], output);
    }
}

#[test]
pub fn eq_limit() {
    for _ in 0..1000 {
        let limit = fastrand::usize(1..MAX_MEMORY_TO_ALLOCATE);
        let input_length = limit;
        let input: Vec<_> = std::iter::repeat_with(|| fastrand::u8(..))
            .take(input_length)
            .collect();
        let cursor = Cursor::new(&input);
        let mut output = Vec::new();
        let result = block_on(cursor.limit_bytes(limit).read_to_end(&mut output));
        assert!(result.is_err());
        assert_eq!(output.len(), limit);
        assert_eq!(input[..limit], output);
    }
}
