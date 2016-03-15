#[macro_use]
extern crate pattern_api_v2_test_support;

searcher_test!(str_searcher_ascii_haystack, "bb", "abbcbbd", double, is exact [
    Reject(0, 1),
    Match (1, 3),
    Reject(3, 4),
    Match (4, 6),
    Reject(6, 7),
]);
searcher_test!(str_searcher_ascii_haystack_seq, "bb", "abbcbbbbd", double, is exact [
    Reject(0, 1),
    Match (1, 3),
    Reject(3, 4),
    Match (4, 6),
    Match (6, 8),
    Reject(8, 9),
]);
searcher_test!(str_searcher_ascii_haystack_ambiguity, "aa", "aaa", reverse, is exact [
    Match (0, 2),
    Reject(2, 3),
], [
    Reject(0, 1),
    Match (1, 3),
]);
searcher_test!(str_searcher_empty_needle_ascii_haystack, "", "abbcbbd", double, is exact [
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
searcher_test!(str_searcher_mulibyte_haystack, " ", "├──", double, is exact [
    Reject(0, 3),
    Reject(3, 6),
    Reject(6, 9),
]);
searcher_test!(str_searcher_empty_needle_mulibyte_haystack, "", "├──", double, is exact [
    Match (0, 0),
    Reject(0, 3),
    Match (3, 3),
    Reject(3, 6),
    Match (6, 6),
    Reject(6, 9),
    Match (9, 9),
]);
searcher_test!(str_searcher_empty_needle_empty_haystack, "", "", double, is exact [
    Match(0, 0),
]);
searcher_test!(str_searcher_nonempty_needle_empty_haystack, "├", "", double, is exact [
]);
searcher_test!(char_searcher_ascii_haystack, 'b', "abbcbbd", double, is exact [
    Reject(0, 1),
    Match (1, 2),
    Match (2, 3),
    Reject(3, 4),
    Match (4, 5),
    Match (5, 6),
    Reject(6, 7),
]);
searcher_test!(char_searcher_mulibyte_haystack, ' ', "├──", double, is exact [
    Reject(0, 3),
    Reject(3, 6),
    Reject(6, 9),
]);
searcher_test!(char_searcher_short_haystack, '\u{1F4A9}', "* \t", double, is exact [
    Reject(0, 1),
    Reject(1, 2),
    Reject(2, 3),
]);

