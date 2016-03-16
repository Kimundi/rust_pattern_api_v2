#[macro_use]
extern crate pattern_api_v2_test_support;
extern crate pattern_api_v2;

pub use pattern_api_v2::slice::Elem;

// &[u8] tests
searcher_test!(slice_searcher_haystack, &b"bb"[..], &b"abbcbbd"[..], double, is exact [
    Reject(0, 1),
    Match (1, 3),
    Reject(3, 4),
    Match (4, 6),
    Reject(6, 7),
]);
searcher_test!(slice_searcher_haystack_seq, &b"bb"[..], &b"abbcbbbbd"[..], double, is exact [
    Reject(0, 1),
    Match (1, 3),
    Reject(3, 4),
    Match (4, 6),
    Match (6, 8),
    Reject(8, 9),
]);
searcher_test!(slice_searcher_haystack_ambiguity, &b"aa"[..], &b"aaa"[..], reverse, is exact [
    Match (0, 2),
    Reject(2, 3),
], [
    Reject(0, 1),
    Match (1, 3),
]);
searcher_test!(slice_searcher_empty_needle_haystack, &b""[..], &b"abbcbbd"[..], double, is exact [
    Match (0, 0),
    Reject(0, 1),
    Match (1, 1),
    Reject(1, 2),
    Match (2, 2),
    Reject(2, 3),
    Match (3, 3),
    Reject(3, 4),
    Match (4, 4),
    Reject(4, 5),
    Match (5, 5),
    Reject(5, 6),
    Match (6, 6),
    Reject(6, 7),
    Match (7, 7),
]);
searcher_test!(slice_searcher_empty_needle_empty_haystack, &b""[..], &b""[..], double, is exact [
    Match(0, 0),
]);
searcher_test!(elem_searcher_haystack, Elem(b'b'), &b"abbcbbd"[..], double, is exact [
    Reject(0, 1),
    Match (1, 2),
    Match (2, 3),
    Reject(3, 4),
    Match (4, 5),
    Match (5, 6),
    Reject(6, 7),
]);

