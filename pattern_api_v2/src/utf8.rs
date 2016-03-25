pub trait CharEq {
    fn matches(&mut self, char) -> bool;
    fn only_ascii(&self) -> bool;
}

impl CharEq for char {
    #[inline]
    fn matches(&mut self, c: char) -> bool { *self == c }

    #[inline]
    fn only_ascii(&self) -> bool { (*self as u32) < 128 }
}

impl<F> CharEq for F where F: FnMut(char) -> bool {
    #[inline]
    fn matches(&mut self, c: char) -> bool { (*self)(c) }

    #[inline]
    fn only_ascii(&self) -> bool { false }
}

impl<'a> CharEq for &'a [char] {
    #[inline]
    fn matches(&mut self, c: char) -> bool {
        self.iter().any(|&m| { let mut m = m; m.matches(c) })
    }

    #[inline]
    fn only_ascii(&self) -> bool {
        self.iter().all(|m| m.only_ascii())
    }
}

pub struct CharEqPattern<C: CharEq>(pub C);

/// Mask of the value bits of a continuation byte
const CONT_MASK: u8 = 0b0011_1111;
/// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte
const TAG_CONT_U8: u8 = 0b1000_0000;

/// Return the initial codepoint accumulator for the first byte.
/// The first byte is special, only want bottom 5 bits for width 2, 4 bits
/// for width 3, and 3 bits for width 4.
#[inline]
fn utf8_first_byte(byte: u8, width: u32) -> u32 { (byte & (0x7F >> width)) as u32 }

/// Return the value of `ch` updated with continuation byte `byte`.
#[inline]
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 { (ch << 6) | (byte & CONT_MASK) as u32 }

/// Checks whether the byte is a UTF-8 continuation byte (i.e. starts with the
/// bits `10`).
#[inline]
fn utf8_is_cont_byte(byte: u8) -> bool { (byte & !CONT_MASK) == TAG_CONT_U8 }

#[inline]
fn unwrap_or_0(opt: Option<u8>) -> u8 {
    match opt {
        Some(byte) => byte,
        None => 0,
    }
}

/// Reads the next code point out of a byte iterator (assuming a
/// UTF-8-like encoding).
pub fn next_code_point<F>(mut next_byte: F) -> Option<char>
    where F: FnMut() -> Option<u8>
{
    // Decode UTF-8
    let x = match next_byte() {
        None => return None,
        Some(next_byte) if next_byte < 128 => return Some(next_byte as char),
        Some(next_byte) => next_byte,
    };

    // Multibyte case follows
    // Decode from a byte combination out of: [[[x y] z] w]
    // NOTE: Performance is sensitive to the exact formulation here
    let init = utf8_first_byte(x, 2);
    let y = unwrap_or_0(next_byte());
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        // [[x y z] w] case
        // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
        let z = unwrap_or_0(next_byte());
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            // [x y z w] case
            // use only the lower 3 bits of `init`
            let w = unwrap_or_0(next_byte());
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    Some(unsafe { ::std::mem::transmute(ch) })
}

/// Reads the last code point out of a byte iterator (assuming a
/// UTF-8-like encoding).
pub fn next_code_point_reverse<F>(mut next_byte_back: F) -> Option<char>
    where F: FnMut() -> Option<u8>
{
    // Decode UTF-8
    let w = match next_byte_back() {
        None => return None,
        Some(next_byte) if next_byte < 128 => return Some(next_byte as char),
        Some(back_byte) => back_byte,
    };

    // Multibyte case follows
    // Decode from a byte combination out of: [x [y [z w]]]
    let mut ch;
    let z = unwrap_or_0(next_byte_back());
    ch = utf8_first_byte(z, 2);
    if utf8_is_cont_byte(z) {
        let y = unwrap_or_0(next_byte_back());
        ch = utf8_first_byte(y, 3);
        if utf8_is_cont_byte(y) {
            let x = unwrap_or_0(next_byte_back());
            ch = utf8_first_byte(x, 4);
            ch = utf8_acc_cont_byte(ch, y);
        }
        ch = utf8_acc_cont_byte(ch, z);
    }
    ch = utf8_acc_cont_byte(ch, w);

    Some(unsafe { ::std::mem::transmute(ch) })
}

pub fn byte_is_char_boundary(b: u8) -> bool {
    b < 128 || b >= 192
}

#[inline]
pub unsafe fn ptr_range_starts_with_valid_utf8(start: *const u8, end: *const u8) -> bool {
    let len = end as usize - start as usize;
    let len = ::std::cmp::min(len, 4);
    let s: &[u8] = ::std::slice::from_raw_parts(start, len);
    match ::std::str::from_utf8(s) {
        Ok(_) => true,
        Err(e) if e.valid_up_to() > 0 => true,
        _ => false,
    }
}

#[test]
fn test_ptr_range_starts_with_valid_utf8() {
    let check = |s: &'static [u8]| unsafe {
        ptr_range_starts_with_valid_utf8(s.as_ptr(), s.as_ptr().offset(s.len() as isize))
    };

    assert!(check(b"abcde"));
    assert!(check(b"abcd"));
    assert!(check(b"abc"));
    assert!(check(b"a"));
    assert!(check(b""));

    assert!(check(b"abcd\xff"));
    assert!(check(b"abcd\xbe"));
    assert!(check(b"abc\xff"));
    assert!(check(b"abc\xbe"));
    assert!(check(b"ab\xff"));
    assert!(check(b"ab\xbe"));
    assert!(check(b"a\xff"));
    assert!(check(b"a\xbe"));

    assert!(!check(b"\xff"));
    assert!(!check(b"\xbe"));
    assert!(!check(b"\xff\xff\xff\xff\xff"));
    assert!(!check(b"\xbe\xbe\xbe\xbe\xbe"));

    assert!(!check(b"\xff1"));
    assert!(!check(b"\xbe1"));
    assert!(!check(b"\xff12"));
    assert!(!check(b"\xbe12"));
    assert!(!check(b"\xff123"));
    assert!(!check(b"\xbe123"));
    assert!(!check(b"\xff1234"));
    assert!(!check(b"\xbe1234"));
}

#[inline]
pub unsafe fn ptr_range_ends_with_valid_utf8(start: *const u8, mut end: *const u8) -> bool {
    let original_end = end;
    // Search for a valid utf8 start sequence in the up to last 4 bytes
    while start != end && (original_end as usize - end as usize) <= 4 {
        end = end.offset(-1);
        // Do a somewhat inefficient chain of operations.
        // This should probably be optimized.

        // If there could be a char here...
        if byte_is_char_boundary(*end) {
            // ... and there is a char here...
            if ptr_range_starts_with_valid_utf8(end, original_end) {
                // ... subtract its length...
                next_code_point(|| {
                    let r = Some(*end);
                    end = end.offset(1);
                    r
                });

                // ... and finally return whether there are invalid bytes in between
                return end == original_end;
            }
        }
    }
    end == original_end
}

#[test]
fn test_ptr_range_ends_with_valid_utf8() {
    let check = |s: &'static [u8]| unsafe {
        ptr_range_ends_with_valid_utf8(s.as_ptr(), s.as_ptr().offset(s.len() as isize))
    };

    assert!(check(b"abcde"));
    assert!(check(b"abcd"));
    assert!(check(b"abc"));
    assert!(check(b"a"));
    assert!(check(b""));

    assert!(!check(b"abcd\xff"));
    assert!(!check(b"abcd\xbe"));
    assert!(!check(b"abc\xff"));
    assert!(!check(b"abc\xbe"));
    assert!(!check(b"ab\xff"));
    assert!(!check(b"ab\xbe"));
    assert!(!check(b"a\xff"));
    assert!(!check(b"a\xbe"));

    assert!(!check(b"\xff"));
    assert!(!check(b"\xbe"));
    assert!(!check(b"\xff\xff\xff\xff\xff"));
    assert!(!check(b"\xbe\xbe\xbe\xbe\xbe"));

    assert!(check(b"\xff1"));
    assert!(check(b"\xbe1"));
    assert!(check(b"\xff12"));
    assert!(check(b"\xbe12"));
    assert!(check(b"\xff123"));
    assert!(check(b"\xbe123"));
    assert!(check(b"\xff1234"));
    assert!(check(b"\xbe1234"));
}

