use std::ffi::{CStr, c_char};

pub unsafe extern "C" fn println_string(string: *const c_char) {
    let cstr = CStr::from_ptr(string);
    println!("{}", cstr.to_str().unwrap())
}

pub extern "C" fn println_int(int: isize) {
    println!("{}", int)
}

pub extern "C" fn assert_int(int: isize, pre: isize) {
    assert_eq!(int, pre)
}
