#![feature(asm)]
#![feature(plugin)]
#![plugin(rust_usdt)]

use std::thread::sleep;
use std::time::Duration;

fn main() {
    let a = 20i32;
    let b = 0.54f32;
    for i in 0..100u8 {
        println!("{}", i);
        sleep(Duration::from_millis(1000));
        static_probe!(provider="foo", name="bar"; i as i64);
    }
}


