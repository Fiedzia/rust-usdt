#![feature(asm)]
#![feature(plugin)]
#![plugin(rust_usdt)]

use std::thread::sleep;
use std::time::Duration;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

fn main() {
    for i in 0..100u8 {
        println!("{}", i);
        let u1: u8 = 45;
        let u2: u8 = 36;
        let s1: String = "abc".to_string();
        sleep(Duration::from_millis(1000));
        let s2:&str = s1.as_str();
        static_probe!(provider="foo", name="bar"; i u8, u1 u8, (OsString::from(&s1).as_os_str()).as_bytes().as_ptr() Ptr);
    }
}


