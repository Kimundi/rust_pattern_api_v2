#[macro_use]
extern crate pattern_api_v2_test_support;
extern crate pattern_api_v2;

pub use std::ffi::{OsStr, OsString};
use std::ops::{Deref, DerefMut};
use std::mem;

pub fn os(s: &[u8]) -> OsStrBuf {
    let s = unsafe {
        mem::transmute::<&[u8], &str>(s)
    };
    OsStrBuf(s.into())
}

pub struct OsStrBuf(OsString);

impl Deref for OsStrBuf {
    type Target = OsStr;
    fn deref(&self) -> &OsStr {
        &*self.0
    }
}

impl DerefMut for OsStrBuf {
    // I'm evil, I know >:)
    #[allow(mutable_transmutes)]
    fn deref_mut(&mut self) -> &mut OsStr {
        unsafe {
            mem::transmute::<&OsStr, &mut OsStr>(&*self.0)
        }
    }
}

searcher_cross_test! {
    test1 {
        double: [
            Reject(0, 1),
            Match (1, 3),
            Reject(3, 4),
            Match (4, 6),
            Reject(6, 7),
        ];
        for:

    //  name,         slice type: input                          pattern
        str,          &str:      "abbcbbd",                      &str:   "bb";
        str_mut,      &mut str:   &mut String::from("abbcbbd"),  &str:   "bb";
        u8_slice,     &[u8]:      b"abbcbbd",                    &[u8]:  b"bb";
        u8_slice_mut, &mut [u8]:  &mut {*b"abbcbbd"},            &[u8]:  b"bb";
        slice,        &[u32]:     &[1,2,2,3,2,2,4],              &[u32]: &[2,2];
        slice_mut,    &mut [u32]: &mut {[1,2,2,3,2,2,4]},        &[u32]: &[2,2];
        slice2,       &[i32]:     &[-1,-2,-2,-3,-2,-2,-4],       &[i32]: &[-2,-2];
        slice2_mut,   &mut [i32]: &mut {[-1,-2,-2,-3,-2,-2,-4]}, &[i32]: &[-2,-2];
        os_str,       &OsStr:     &os(b"abb\xffbbd"),            &OsStr: &os(b"bb");
        os_str_mut,   &mut OsStr: &mut os(b"abb\xffbbd"),        &OsStr: &os(b"bb");
        os_str2,      &OsStr:     &os(b"abb\xffbbd"),            &str:   "bb";
        os_str_mut2,  &mut OsStr: &mut os(b"abb\xffbbd"),        &str:   "bb";
    }
}
