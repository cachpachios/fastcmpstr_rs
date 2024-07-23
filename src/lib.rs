#![warn(clippy::all, clippy::pedantic)]

use core::slice;
use core::{alloc::Layout, fmt::Display, ptr::null_mut};

const PREFIX_LENGTH: usize = 10;
type CapacityOffsetType = u16;
type LenType = u32;

#[repr(C)]
pub struct Str {
    len: LenType,
    prefix: [u8; PREFIX_LENGTH],
    capacity_offset: CapacityOffsetType,
    suffix: *mut u8, // len + capacity_offset
}

impl Str {
    #[inline]
    #[must_use]
    pub fn from(str: &str) -> Self {
        let bytes = str.as_bytes();
        let _len = bytes.len();
        debug_assert!(
            _len < LenType::MAX as usize,
            "Size of string is above LenType limit."
        );
        let len = _len as LenType;
        let mut prefix: [u8; PREFIX_LENGTH] = [0; PREFIX_LENGTH];
        let mut suffix: *mut u8 = null_mut();

        let prefix_len = _len.min(PREFIX_LENGTH);
        prefix[..prefix_len].copy_from_slice(&bytes[..prefix_len]);

        if len > PREFIX_LENGTH as LenType {
            let ptr_len = len as usize - PREFIX_LENGTH;
            unsafe {
                suffix = std::alloc::alloc(Layout::array::<u8>(ptr_len).unwrap()); //TODO: Unsafe unwrap?
                core::ptr::copy(bytes.as_ptr().add(PREFIX_LENGTH), suffix, ptr_len);
            }
        }

        Self {
            len,
            prefix,
            capacity_offset: 0,
            suffix,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    #[must_use]
    pub fn capacity(&self) -> usize {
        let prefix_extra_capactiy = PREFIX_LENGTH - PREFIX_LENGTH.min(self.len as usize);
        self.len as usize + self.capacity_offset as usize + prefix_extra_capactiy
    }

    #[inline]
    #[must_use]
    pub fn starts_with(&self, other: &Str) -> bool {
        if other.len > self.len {
            return false;
        }
        let other_len = other.len();

        if !self
            .prefix
            .starts_with(&other.prefix[..PREFIX_LENGTH.min(other.len())])
        {
            return false;
        }

        if self.len <= PREFIX_LENGTH as LenType || other.len <= PREFIX_LENGTH as LenType {
            return true;
        }

        unsafe {
            let self_ptr_len = self.len() - PREFIX_LENGTH;
            let self_suffix = slice::from_raw_parts(self.suffix, self_ptr_len);
            let other_ptr_len = other_len - PREFIX_LENGTH;
            let other_suffix = slice::from_raw_parts(other.suffix, other_ptr_len);

            self_suffix.starts_with(other_suffix)
        }
    }

    fn reserve(&mut self, request: usize) {
        let prefix_extra_capactiy = PREFIX_LENGTH - PREFIX_LENGTH.min(self.len as usize);
        let current_extra_capacity = self.capacity_offset as usize + prefix_extra_capactiy;
        if current_extra_capacity >= request {
            return;
        }
        debug_assert!(
            request < CapacityOffsetType::MAX as usize,
            "Reserve is above capacity limit."
        );
        let new_cap_offset = request - current_extra_capacity;
        let new_ptr_len = self.len as usize + new_cap_offset;

        let new_mem;

        unsafe {
            new_mem = std::alloc::alloc(Layout::array::<u8>(new_ptr_len).unwrap()); //TODO: Unsafe unwrap?

            if self.len > PREFIX_LENGTH as u32 {
                core::ptr::copy(self.suffix, new_mem, self.len as usize - PREFIX_LENGTH);
            }
            if !self.suffix.is_null() {
                let old_total_cap = self.len + self.capacity_offset - PREFIX_LENGTH;
                std::alloc::dealloc(ptr, Layout::array::<u8>(old_total_cap)).unwrap()
            }
        }
        self.suffix = new_mem;
        self.capacity_offset = new_cap_offset as u16;
        // if old_ptr:
    }
}

pub trait StartsWithStr {
    fn starts_with(&self, other: &str) -> bool;
}

impl StartsWithStr for Str {
    #[inline]
    fn starts_with(&self, other: &str) -> bool {
        if other.len() > self.len() {
            return false;
        }
        let other_bytes = other.as_bytes();

        if !self
            .prefix
            .starts_with(&other_bytes[..PREFIX_LENGTH.min(other.len())])
        {
            return false;
        }

        unsafe {
            let self_ptr_len = self.len() - PREFIX_LENGTH;
            let self_suffix = slice::from_raw_parts(self.suffix, self_ptr_len);

            self_suffix.starts_with(&other_bytes[PREFIX_LENGTH..])
        }
    }
}

impl core::ops::Index<usize> for Str {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &u8 {
        assert!(!(index >= self.len()), "Indexing outside of string length!");

        if index >= PREFIX_LENGTH {
            unsafe { return &*self.suffix.add(index - PREFIX_LENGTH) }
        }
        &self.prefix[index]
    }
}

impl PartialEq for Str {
    #[inline]
    fn eq(&self, other: &Str) -> bool {
        if self.len != other.len {
            return false;
        }

        if self.prefix != other.prefix {
            return false;
        }

        if self.len > PREFIX_LENGTH as LenType {
            let ptr_len = self.len as usize - PREFIX_LENGTH;
            unsafe {
                let a = slice::from_raw_parts(self.suffix, ptr_len);
                let b = slice::from_raw_parts(other.suffix, ptr_len);
                return a == b; // TODO: Perf maybe more efficient to handcraft.
            }
        }
        true
    }
}
impl Display for Str {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let prefix_str;
        let mut suffix_str = "";
        unsafe {
            prefix_str =
                core::str::from_utf8_unchecked(&self.prefix[0..PREFIX_LENGTH.min(self.len())]);
            if self.len > PREFIX_LENGTH as LenType {
                let ptr_len = self.len as usize - PREFIX_LENGTH;
                suffix_str =
                    core::str::from_utf8_unchecked(slice::from_raw_parts(self.suffix, ptr_len));
            }
        }
        write!(f, "{prefix_str}{suffix_str}")
    }
}

impl core::fmt::Debug for Str {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    const LONG_STR: &str = "this is a longer string that will primarly be in the suffix";
    const LONG_STR2: &str = "let me tell you a story when unsafe went very wrong...";

    #[test]
    fn test_from_empty() {
        let s = Str::from("");

        assert_eq!(s.len, 0);
        assert!(s.is_empty());
        assert_eq!(s.capacity_offset, 0);
        assert_eq!(s.prefix, [0; PREFIX_LENGTH]);
        assert_eq!(s.suffix, null_mut());
    }

    #[test]
    fn test_from_no_suffix() {
        let s = Str::from("abc");

        assert_eq!(s.len, 3);
        assert!(!s.is_empty());
        assert_eq!(s.capacity_offset, 0);
        let mut expected_prefix: [u8; PREFIX_LENGTH] = [0; PREFIX_LENGTH];
        expected_prefix[..3].clone_from_slice("abc".as_bytes());
        assert_eq!(s.prefix, expected_prefix);
        assert_eq!(s.suffix, null_mut());
    }

    #[test]
    fn test_from_with_suffix() {
        let s = Str::from(LONG_STR);

        assert_eq!(s.len, LONG_STR.len() as LenType);
        assert_eq!(s.capacity_offset, 0);
        assert_eq!(s.prefix, LONG_STR.as_bytes()[..PREFIX_LENGTH]);
        assert_ne!(s.suffix, null_mut());

        let ptr_len = s.len as usize - PREFIX_LENGTH;

        let suffix_slice;
        unsafe {
            suffix_slice = slice::from_raw_parts(s.suffix, ptr_len);
        }
        let suffix_str = core::str::from_utf8(suffix_slice).unwrap();
        assert_eq!(suffix_str, &LONG_STR[PREFIX_LENGTH..]);
    }

    #[test]
    fn test_indexing() {
        assert_eq!(Str::from("test")[2], "test".as_bytes()[2]);

        for i in 0..LONG_STR.len() {
            assert_eq!(Str::from(LONG_STR)[i], LONG_STR.as_bytes()[i]);
        }
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

    fn rand_str(rand_len: usize) -> String {
        let r: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(rand_len)
            .map(char::from)
            .collect();

        return r;
    }

    #[test]
    fn test_eq_rand() {
        for i in 0..100 {
            for _ in 0..100 {
                let s = rand_str(i);
                let a = Str::from(&s);
                let a2 = Str::from(&format!("{}^", s));
                let b = Str::from(&s);
                let b2 = Str::from(&format!("{}_", s));

                assert_eq!(a, b);
                assert_ne!(a2, b2);
            }
        }
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Str::from("a").to_string(), "a".to_string());
        assert_eq!(Str::from("ab").to_string(), "ab".to_string());
        assert_eq!(Str::from("abc").to_string(), "abc".to_string());
        assert_eq!(Str::from(LONG_STR).to_string(), LONG_STR.to_string());
    }

    #[test]
    fn test_starts_with_other() {
        let a = Str::from(LONG_STR);
        let b = Str::from(LONG_STR2);
        let a_short = Str::from(&LONG_STR[..PREFIX_LENGTH]);
        let b_short = Str::from(&LONG_STR2[..PREFIX_LENGTH]);
        let a_shorter = Str::from(&LONG_STR[..PREFIX_LENGTH - 2]);

        assert!(a.starts_with(&a));
        assert!(b.starts_with(&b));
        assert!(!a.starts_with(&b));
        assert!(!b.starts_with(&a));

        assert!(a.starts_with(&a_short));
        assert!(b.starts_with(&b_short));
        assert!(a_short.starts_with(&a_short));
        assert!(b_short.starts_with(&b_short));
        assert!(!a.starts_with(&b_short));
        assert!(!b.starts_with(&a_short));

        assert!(a.starts_with(&a_shorter));
        assert!(a_short.starts_with(&a_shorter));
        assert!(a_shorter.starts_with(&a_shorter))
    }
}
