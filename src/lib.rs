use futures_lite::AsyncRead;
use std::{
    error::Error,
    fmt::Display,
    io::{ErrorKind, Result},
    pin::Pin,
    task::{ready, Context, Poll},
};

pin_project_lite::pin_project! {
    /// [`AsyncRead`] length limiter
    ///
    /// Protects against a certain class of denial-of-service attacks wherein long chunked bodies are
    /// uploaded to web services.
    ///
    /// The number of bytes will never be more than the provided byte limit. If the byte limit is
    /// exactly the length of the contained AsyncRead, it is considered an error.
    ///
    /// # Errors
    ///
    /// This will return an error if the underlying AsyncRead does so or if the read length meets (or
    /// would exceed) the provided length limit. The returned [`std::io::Error`] will have an error kind
    /// of [`ErrorKind::InvalidData`] and a contained error of [`LengthLimitExceeded`].
    pub struct LengthLimit<T> {
        #[pin]
        reader:  T,
        bytes_remaining: usize,
    }
}

impl<T> LengthLimit<T> {
    pub fn new(reader: T, max_bytes: usize) -> Self {
        Self {
            reader,
            bytes_remaining: max_bytes,
        }
    }
}

/// A unit error that represents a length overflow.
///
/// Contains no further information
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LengthLimitExceeded;
impl Display for LengthLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Length limit exceeded")
    }
}
impl Error for LengthLimitExceeded {}
impl From<LengthLimitExceeded> for std::io::Error {
    fn from(value: LengthLimitExceeded) -> Self {
        Self::new(ErrorKind::InvalidInput, value)
    }
}

impl<T: AsyncRead> AsyncRead for LengthLimit<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let projection = self.project();
        let reader = projection.reader;
        let bytes_remaining = *projection.bytes_remaining;

        if bytes_remaining == 0 {
            return Poll::Ready(Err(LengthLimitExceeded.into()));
        }

        if bytes_remaining < buf.len() {
            buf = &mut buf[..bytes_remaining];
        }

        let new_bytes = ready!(reader.poll_read(cx, buf))?;
        *projection.bytes_remaining = bytes_remaining.saturating_sub(new_bytes);
        Poll::Ready(Ok(new_bytes))
    }
}

/// Extension trait to add length limiting behavior to any AsyncRead
///
/// Full explanation of the behavior at [`LengthLimit`]
pub trait LengthLimitExt: Sized + AsyncRead {
    /// Applies a LengthLimit to self with an exclusive maxiumum of `max_bytes` bytes
    fn limit_bytes(self, max_bytes: usize) -> LengthLimit<Self> {
        LengthLimit::new(self, max_bytes)
    }

    /// Applies a LengthLimit to self with an exclusive maxiumum of `max_kb` kilobytes (defined as
    /// 1024 bytes)
    fn limit_kb(self, max_kb: usize) -> LengthLimit<Self> {
        self.limit_bytes(max_kb * 1024)
    }

    /// Applies a LengthLimit to self with an exclusive maxiumum of `max_mb` megabytes (defined as
    /// 1024 kilobytes, or 1,048,576 bytes)
    fn limit_mb(self, max_mb: usize) -> LengthLimit<Self> {
        self.limit_kb(max_mb * 1024)
    }

    /// Applies a LengthLimit to self with an exclusive maxiumum of `max_gb` kilobytes (defined as
    /// 1024 megabytes, or 1,073,741,824 bytes)
    fn limit_gb(self, max_gb: usize) -> LengthLimit<Self> {
        self.limit_mb(max_gb * 1024)
    }
}

impl<T> LengthLimitExt for T where T: AsyncRead + Unpin {}
