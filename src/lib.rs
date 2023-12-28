use core::fmt;

// const ELIPSIS_SIZE: usize = 3;

pub struct TruncatingFmtBuffer<const N: usize> {
    buffer: [u8; N],
    used: usize,
    truncated: bool,
}

impl<const N: usize> TruncatingFmtBuffer<N> {
    // #[allow(unused)]
    // const fn assert_size() {
    //     if N <= ELIPSIS_SIZE {
    //         panic!("smaller than ellipsis size");
    //     }
    // }

    pub fn new() -> Self {
        TruncatingFmtBuffer {
            buffer: [0u8; N],
            used: 0,
            truncated: false,
        }
    }

    pub fn as_str(&mut self) -> (&str, bool) {
        debug_assert!(self.used <= self.buffer.len());
        use core::str::from_utf8_unchecked;
        (
            unsafe { from_utf8_unchecked(&self.buffer[..self.used]) },
            self.truncated,
        )
    }
}

impl<const N: usize> fmt::Write for TruncatingFmtBuffer<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining_buf = &mut self.buffer[self.used..];
        let raw_s = s.as_bytes();

        let raw_s_len = raw_s.len();
        let remaining_len = remaining_buf.len();

        let bytes_written = if remaining_len < raw_s_len {
            self.truncated = true;
            match s.char_indices().rfind(|&(ind, _char)| ind <= remaining_len) {
                None => {
                    // noop, truncating all reftover chars
                    return Ok(());
                }
                Some((ind, _cahr)) => ind,
            }
        } else {
            raw_s_len
        };
        remaining_buf[..bytes_written].copy_from_slice(&raw_s[..bytes_written]);
        self.used += bytes_written;
        Ok(())
    }
}

// pub fn show<'a>(buffer: &'a mut [u8], args: fmt::Arguments) -> Result<&'a str, fmt::Error> {
//     let mut w = FmtBuffer::new(buffer);
//     fmt::write(&mut w, args)?;
//     w.as_str().ok_or(fmt::Error)
// }

#[cfg(test)]
mod tests {
    use core::fmt::Write;
    use core::write;

    use crate::TruncatingFmtBuffer;

    #[test]
    pub fn test() {
        let mut buffer = TruncatingFmtBuffer::<30>::new();

        write!(buffer, "{} - 0x{:x}", 42, 42).expect("truncated fmt buffer error");

        assert_eq!(("42 - 0x2a", false), buffer.as_str());
    }
    #[test]
    pub fn test_longer() {
        let mut buffer = TruncatingFmtBuffer::<30>::new();

        write!(buffer, "{} - 0x{:x}", 4000, 4001).expect("truncated fmt buffer error");

        assert_eq!(("4000 - 0xfa1", false), buffer.as_str());
    }

    #[test]
    pub fn test_longer_trunc() {
        let mut buffer = TruncatingFmtBuffer::<30>::new();

        write!(buffer, "long: {} - 0x{:x}", 400000, 400100).expect("truncated fmt buffer error");

        assert_eq!(("long: 400000 - 0x61ae4", false), buffer.as_str());
    }

    #[test]
    pub fn test_too_long() {
        let mut buffer = TruncatingFmtBuffer::<30>::new();

        write!(buffer, "toooooo long:    {} - 0x{:x}", 400000, 400100)
            .expect("truncated fmt buffer error");

        assert_eq!(("toooooo long:    400000 - 0x61", true), buffer.as_str());
    }

    #[test]
    pub fn test_too_long_over_the_end() {
        let mut buffer = TruncatingFmtBuffer::<30>::new();

        write!(buffer, "toooooo long").expect("truncated fmt buffer error");
        assert_eq!(("toooooo long", false), buffer.as_str());
        write!(buffer, ":    {} - 0x{:x}", 400000, 400100).expect("truncated fmt buffer error");

        assert_eq!(("toooooo long:    400000 - 0x61", true), buffer.as_str());
        write!(buffer, "some more").expect("truncated fmt buffer error");
        assert_eq!(("toooooo long:    400000 - 0x61", true), buffer.as_str());
    }
}
