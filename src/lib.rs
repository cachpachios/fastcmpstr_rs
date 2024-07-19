use core::slice;
use std::{alloc::Layout, fmt::Display, ptr::null_mut};

const PREFIX_LENGTH: usize = 3;
pub struct Str {
    len: u32,
    prefix: [u8; PREFIX_LENGTH],
    capacity_offset: u8,
    suffix: *mut u8, // len + capacity_offset
}

impl Str {
    pub fn from(s: &str) -> Self {
        let bytes = s.as_bytes();
        let _len = bytes.len();
        debug_assert!(
            _len < u32::MAX as usize,
            "Size of string is above u32 limit."
        );
        let len = _len as u32;
        let mut prefix: [u8; PREFIX_LENGTH] = [0; PREFIX_LENGTH];
        let mut suffix: *mut u8 = null_mut();

        for i in 0.._len.min(PREFIX_LENGTH) {
            prefix[i] = bytes[i]; //TODO replace with non-runtime checked version
        }

        if len > PREFIX_LENGTH as u32 {
            let ptr_len = len as usize - PREFIX_LENGTH;
            unsafe {
                suffix = std::alloc::alloc(Layout::array::<u8>(ptr_len).unwrap()); //TODO: Unsafe unwrap?
                std::ptr::copy(bytes.as_ptr().add(PREFIX_LENGTH), suffix, ptr_len);
            }
        }

        Self {
            len,
            prefix,
            capacity_offset: 0,
            suffix,
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn capacity(&self) -> usize {
        self.len as usize + self.capacity_offset as usize
    }

    pub fn to_string(&self) -> String {
        let len = self.len();
        // We allocate atleast PREFIX_LENGTH to simplify the number of conditions
        let capacity = len.max(PREFIX_LENGTH);

        let buf: *mut u8;
        let s;
        unsafe {
            buf = std::alloc::alloc(Layout::array::<u8>(capacity).unwrap());
            *buf = self.prefix[0];
            *buf.add(1) = self.prefix[1];
            *buf.add(2) = self.prefix[2];
            if len > 3 {
                let ptr_len = len - PREFIX_LENGTH;
                std::ptr::copy(self.suffix, buf.add(PREFIX_LENGTH), ptr_len);
            }
            s = String::from_raw_parts(buf, len, capacity);
        }
        s
    }
}

impl PartialEq for Str {
    fn eq(&self, other: &Str) -> bool {
        if self.len != other.len {
            return false;
        }

        for i in 0..self.len.min(PREFIX_LENGTH as u32) {
            if self.prefix[i as usize] != other.prefix[i as usize] {
                return false;
            }
        }

        if self.len > 3 {
            let ptr_len = self.len as usize - PREFIX_LENGTH;
            unsafe {
                let a = slice::from_raw_parts(self.suffix, ptr_len);
                let b = slice::from_raw_parts(self.suffix, ptr_len);
                return a == b; // TODO: Perf maybe more efficient to handcraft.
            }
        }
        true
    }
}
impl Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix_str;
        let mut suffix_str = "";
        unsafe {
            prefix_str =
                std::str::from_utf8_unchecked(&self.prefix[0..PREFIX_LENGTH.min(self.len())]);
            if self.len > PREFIX_LENGTH as u32 {
                let ptr_len = self.len as usize - PREFIX_LENGTH;
                suffix_str =
                    std::str::from_utf8_unchecked(slice::from_raw_parts(self.suffix, ptr_len));
            }
        }
        write!(f, "{}{}", prefix_str, suffix_str)
    }
}

impl std::fmt::Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\" (len={}, cap={}, stack_size={}, heap_size={})",
            self,
            self.len,
            self.len as usize + self.capacity_offset as usize,
            PREFIX_LENGTH,
            0.max(self.len as usize + self.capacity_offset as usize - PREFIX_LENGTH),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LONG_STR: &str = "this is a longer string that will primarly be in the suffix";
    const LONG_STR2: &str = "let me tell you a story when unsafe went very wrong...";

    #[test]
    fn test_from_empty() {
        let s = Str::from("");

        assert_eq!(s.len, 0);
        assert_eq!(s.capacity_offset, 0);
        assert_eq!(s.prefix, [0, 0, 0]);
        assert_eq!(s.suffix, null_mut());
    }

    #[test]
    fn test_from_no_suffix() {
        let s = Str::from("abc");

        assert_eq!(s.len, 3);
        assert_eq!(s.capacity_offset, 0);
        assert_eq!(s.prefix, "abc".as_bytes());
        assert_eq!(s.suffix, null_mut());
    }

    #[test]
    fn test_from_with_suffix() {
        let s = Str::from(LONG_STR);

        assert_eq!(s.len, LONG_STR.len() as u32);
        assert_eq!(s.capacity_offset, 0);
        assert_eq!(s.prefix, LONG_STR.as_bytes()[..PREFIX_LENGTH]);
        assert_ne!(s.suffix, null_mut());

        let ptr_len = s.len as usize - PREFIX_LENGTH;

        let suffix_slice;
        unsafe {
            suffix_slice = slice::from_raw_parts(s.suffix, ptr_len);
        }
        let suffix_str = std::str::from_utf8(suffix_slice).unwrap();
        assert_eq!(suffix_str, &LONG_STR[PREFIX_LENGTH..]);
    }

    #[test]
    fn test_eq_no_suffix() {
        let a = Str::from("abc");
        let b = Str::from("dbc");

        assert_eq!(a, a);
        assert_eq!(b, b);
        assert_ne!(a, b);
    }

    #[test]
    fn test_eq_with_suffix() {
        let a = Str::from(LONG_STR);
        let b = Str::from(LONG_STR2);

        assert_eq!(a, a);
        assert_eq!(b, b);
        assert_ne!(a, b);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Str::from("a").to_string(), "a".to_string());
        assert_eq!(Str::from("ab").to_string(), "ab".to_string());
        assert_eq!(Str::from("abc").to_string(), "abc".to_string());
        assert_eq!(Str::from(LONG_STR).to_string(), LONG_STR.to_string());
    }
}
