use crate::{Error, Result};
use memchr::memmem::Finder;
use memchr::{memchr, memrchr};
use once_cell::unsync::Lazy;

pub fn split_once(input: &str, delim: u8, offset: usize) -> Result<(&str, &str)> {
    debug_assert!(delim < 128); // only basic ascii
    debug_assert!(offset <= 1);
    let end = memchr(delim, input.as_bytes()).ok_or(Error::Incomplete)?;
    // safety, memchr returns indices that are inside the input length and we only split on ascii
    Ok(unsafe {
        (
            input.get_unchecked(..end),
            input.get_unchecked(end + offset..),
        )
    })
}

thread_local! {
    static SUBJECT_END_FINDER: Lazy<Finder<'static>> = Lazy::new(|| Finder::new(br#">""#));
}

pub fn split_subject_end<'a>(input: &'a str, offset: usize) -> Result<(&'a str, &'a str)> {
    let start_offset = 1;
    let end_offset = start_offset + offset;
    debug_assert!(offset <= 2);
    let end = SUBJECT_END_FINDER
        .with(|finder| finder.find(input.as_bytes()))
        .ok_or(Error::Incomplete)?;
    // safety, memchr returns indices that are inside the input length and we only split on ascii
    Ok(unsafe {
        (
            input.get_unchecked(..end + start_offset),
            input.get_unchecked(end + end_offset..),
        )
    })
}

pub fn take_until(input: &str, delim: u8) -> (&str, &str) {
    debug_assert!(delim < 128); // only basic ascii
    if let Some(end) = memchr(delim, input.as_bytes()) {
        // safety, memchr returns indices that are inside the input length and we only split on ascii
        unsafe { (input.get_unchecked(end..), input.get_unchecked(..end)) }
    } else {
        ("", input)
    }
}

pub fn skip(input: &str, count: usize) -> Result<&str> {
    input.get(count..).ok_or(Error::Incomplete)
}

pub fn skip_matches(input: &str, char: u8) -> (&str, bool) {
    if input.as_bytes().get(0) == Some(&char) {
        // safety, we verified that the input has a length of at least 1
        (unsafe { input.get_unchecked(1..) }, true)
    } else {
        (input, false)
    }
}

pub fn find_between_end(input: &str, start: u8, end: u8) -> Option<&str> {
    debug_assert!(start < 128 && end < 128); // only basic ascii
    let bytes = input.as_bytes();
    let end = memrchr(end, bytes)?;
    // safety, memchr returns indices that are inside the input
    let start = memrchr(start, unsafe { &bytes.get_unchecked(0..end) })?;
    // safety, memchr returns indices that are inside the input length and we only split on ascii
    Some(unsafe { input.get_unchecked((start + 1)..end) })
}

#[test]
fn test_find_between_end() {
    assert_eq!(Some("foo"), find_between_end("asd[foo]bar", b'[', b']'));
    assert_eq!(None, find_between_end("asd]foo[bar", b'[', b']'));
}
