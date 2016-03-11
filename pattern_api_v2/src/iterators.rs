use super::*;

pub fn match_indices<H, P>(haystack: H, pattern: P) -> Vec<(usize, H)>
    where H: SearchCursors,
            P: Pattern<H>,
{
    let mut searcher = pattern.into_searcher(haystack);
    let mut ret = vec![];

    while let Some((begin, end)) = searcher.next_match() {
        let haystack = searcher.haystack();
        unsafe {
            let offset = H::offset_from_front(haystack, begin);
            let slice = H::range_to_self(haystack, begin, end);

            ret.push((offset, slice));
        }
    }

    ret
}

#[test]
fn test_match_indices() {
    assert_eq!(match_indices("banana", 'a'),
                vec![(1, "a"), (3, "a"), (5, "a")]);

    let mut slice = &mut {*b"banana"}[..];

    {
        let match_indices = match_indices(&mut*slice, slice::Ascii(b'a'));

        assert_eq!(match_indices.iter().map(|x| x.0).collect::<Vec<_>>(),
                    vec![1, 3, 5]);

        for m in match_indices {
            m.1[0] = b'i';
        }
    }

    assert_eq!(slice, b"binini");
}

pub fn split<H, P>(haystack: H, pattern: P) -> Vec<H>
    where H: SearchCursors,
            P: Pattern<H>,
{
    let mut searcher = pattern.into_searcher(haystack);
    let mut ret = vec![];

    let haystack = searcher.haystack();

    let mut last_end = Some(H::cursor_at_front(haystack));

    while let Some((begin, end)) = searcher.next_match() {
        if let Some(last_end) = last_end {
            unsafe {
                let slice = H::range_to_self(haystack, last_end, begin);
                ret.push(slice);
            }
        }
        last_end = Some(end);
    }

    if let Some(last_end) = last_end {
        unsafe {
            let end = H::cursor_at_back(haystack);
            let slice = H::range_to_self(haystack, last_end, end);
            ret.push(slice);
        }
    }

    ret
}

#[test]
fn test_split() {
    assert_eq!(split("hangman", 'a'),
                vec!["h", "ngm", "n"]);

    let mut slice = &mut {*b"hangman"}[..];

    {
        let split = split(&mut*slice, slice::Ascii(b'a'));

        for m in split {
            for byte in m {
                *byte = b'-';
            }
        }
    }

    assert_eq!(slice, b"-a---a-");
}
