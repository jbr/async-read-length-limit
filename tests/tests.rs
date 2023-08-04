use async_read_length_limit::{LengthLimit, LengthLimitExt};
use futures_lite::{future::block_on, io::Cursor, AsyncReadExt};

const MAX_MEMORY_TO_ALLOCATE: usize = 1024 * 1024;
const ITERATIONS: usize = 1000;

#[test]
pub fn under_limit() {
    block_on(async {
        let data: Vec<_> = std::iter::repeat_with(|| fastrand::u8(..))
            .take(MAX_MEMORY_TO_ALLOCATE)
            .collect();

        for _ in 0..ITERATIONS {
            let limit = fastrand::usize(2..MAX_MEMORY_TO_ALLOCATE);
            let input_length = fastrand::usize(1..limit);
            let input = &data[..input_length];
            let cursor = Cursor::new(input);
            let mut output = Vec::new();
            let result = cursor.limit_bytes(limit).read_to_end(&mut output).await;
            assert_eq!(input_length, result.unwrap());
            assert_eq!(input_length, output.len());
            assert_eq!(output, input);
        }
    });
}

#[test]
pub fn benchmark_comparison_to_just_async_read() {
    block_on(async {
        // this does not test any of this crate's code, but should be roughly comparable
        // to the other tests
        let data: Vec<_> = std::iter::repeat_with(|| fastrand::u8(..))
            .take(MAX_MEMORY_TO_ALLOCATE)
            .collect();

        for _ in 0..ITERATIONS {
            let input_length = fastrand::usize(1..MAX_MEMORY_TO_ALLOCATE);
            let input = &data[..input_length];
            let mut cursor = Cursor::new(input);
            let mut output = Vec::new();
            let result = cursor.read_to_end(&mut output).await;
            assert_eq!(input_length, result.unwrap());
            assert_eq!(input_length, output.len());
            assert_eq!(output, input);
        }
    });
}

#[test]
pub fn over_limit() {
    block_on(async {
        let data: Vec<_> = std::iter::repeat_with(|| fastrand::u8(..))
            .take(MAX_MEMORY_TO_ALLOCATE)
            .collect();

        for _ in 0..ITERATIONS {
            let limit = fastrand::usize(1..MAX_MEMORY_TO_ALLOCATE - 1);
            let input_length = fastrand::usize(limit..MAX_MEMORY_TO_ALLOCATE);
            let input = &data[..input_length];
            let cursor = Cursor::new(input);
            let mut output = Vec::new();
            let result = cursor.limit_bytes(limit).read_to_end(&mut output).await;
            assert!(result.is_err());
            assert_eq!(output.len(), limit);
            assert_eq!(output, &input[..limit]);
        }
    });
}

#[test]
pub fn eq_limit() {
    block_on(async {
        let data: Vec<_> = std::iter::repeat_with(|| fastrand::u8(..))
            .take(MAX_MEMORY_TO_ALLOCATE)
            .collect();

        for _ in 0..ITERATIONS {
            let limit = fastrand::usize(1..MAX_MEMORY_TO_ALLOCATE);
            let input_length = limit;
            let input = &data[..input_length];
            let cursor = Cursor::new(input);
            let mut output = Vec::new();
            let result = cursor.limit_bytes(limit).read_to_end(&mut output).await;
            assert!(result.is_err());
            assert_eq!(output.len(), limit);
            assert_eq!(output, &input[..limit]);
        }
    });
}

#[test]
pub fn unit_conversions() {
    block_on(async {
        assert_eq!(Cursor::new(b"").limit_kb(1).bytes_remaining(), 1024);
        assert_eq!(Cursor::new(b"").limit_mb(1).bytes_remaining(), 1024 * 1024);
        assert_eq!(
            Cursor::new(b"").limit_gb(1).bytes_remaining(),
            1024 * 1024 * 1024
        );
    });
}

#[test]
pub fn other_interfaces() {
    block_on(async {
        let cursor = Cursor::new(b"b");
        let length_limit = LengthLimit::new(cursor, 100);
        assert_eq!(
            length_limit.bytes_remaining(),
            length_limit.clone().bytes_remaining()
        );
        assert_eq!(
            r#"LengthLimit {
    reader: Cursor {
        inner: Cursor {
            inner: [
                98,
            ],
            pos: 0,
        },
    },
    bytes_remaining: 100,
}"#,
            &format!("{length_limit:#?}")
        );
        assert_eq!(length_limit.as_ref().get_ref(), &b"b");
        assert_eq!(length_limit.into_inner().into_inner(), b"b");
    });
}

#[test]
pub fn error() {
    block_on(async {
        let cursor = Cursor::new(b"these are the data");
        let mut output = Vec::new();
        let result = cursor.limit_bytes(5).read_to_end(&mut output).await;
        let err = result.unwrap_err();
        assert_eq!("Length limit exceeded", err.to_string());
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    });
}
