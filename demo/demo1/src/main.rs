#![feature(asm)]
#![feature(plugin)]
#![plugin(rust_usdt)]

use std::thread::sleep;
use std::time::Duration;

fn main() {
    let a = 0i64;
    let b = 1i64;
    for i in 0..100 {
        println!("{}", i);
        sleep(Duration::from_millis(1000));
        static_probe!(provider="foo", name="bar"; (i as i64));
    }
}


