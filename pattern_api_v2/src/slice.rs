use super::*;
impl<'a> SearchCursors for &'a mut [u8] {
    type Haystack = (*mut u8, *mut u8);
    type Cursor = *mut u8;

    fn offset_from_front(haystack: Self::Haystack,
                         begin: Self::Cursor) -> usize {
        begin as usize - haystack.0 as usize
    }

    unsafe fn range_to_self(_: Self::Haystack,
                            start: Self::Cursor,
                            end: Self::Cursor) -> Self {
        ::std::slice::from_raw_parts_mut(start,
            end as usize - start as usize)
    }
    fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor {
        hs.0
    }
    fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor {
        hs.1
    }
    fn into_haystack(self) -> Self::Haystack {
        unsafe {
            (self.as_mut_ptr(), self.as_mut_ptr().offset(self.len() as isize))
        }
    }
}

pub struct Ascii(pub u8);

pub struct AsciiSearcher<'a> {
    haystack: (*mut u8, *mut u8),
    start: *mut u8,
    end: *mut u8,
    ascii: u8,
    _marker: ::std::marker::PhantomData<&'a mut [u8]>
}

unsafe impl<'a> Searcher<&'a mut [u8]> for AsciiSearcher<'a> {
    fn haystack(&self) -> (*mut u8, *mut u8) {
        self.haystack
    }

    fn next_match(&mut self) -> Option<(*mut u8, *mut u8)> {
        while self.start != self.end {
            unsafe {
                let p = self.start;
                self.start = self.start.offset(1);

                if *p == self.ascii {
                    return Some((p, self.start));
                }
            }
        }
        None
    }

    fn next_reject(&mut self) -> Option<(*mut u8, *mut u8)> {
        while self.start != self.end {
            unsafe {
                let p = self.start;
                self.start = self.start.offset(1);

                if *p != self.ascii {
                    return Some((p, self.start));
                }
            }
        }
        None
    }
}

impl<'a> Pattern<&'a mut [u8]> for Ascii {
    type Searcher = AsciiSearcher<'a>;

    fn into_searcher(self, haystack: &'a mut [u8]) -> Self::Searcher {
        let begin = haystack.as_mut_ptr();
        let end = unsafe {
            haystack.as_mut_ptr().offset(haystack.len() as isize)
        };

        AsciiSearcher {
            haystack: (begin, end),
            start: begin,
            end: end,
            ascii: self.0,
            _marker: ::std::marker::PhantomData,
        }
    }

    fn is_prefix_of(self, haystack: &'a mut [u8]) -> bool {
        haystack
            .get(0)
            .map(|&b| b == self.0)
            .unwrap_or(false)
    }

    fn is_suffix_of(self, haystack: &'a mut [u8]) -> bool
        where Self::Searcher: ReverseSearcher<&'a mut [u8]> {
        haystack
            .get(haystack.len() - 1)
            .map(|&b| b == self.0)
            .unwrap_or(false)
    }
}
