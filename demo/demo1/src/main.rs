#![feature(asm)]
#![feature(plugin)]
#![plugin(rust_usdt)]


fn main() {
    let a = 0i64;
    let b = 1i64;
    static_probe!(provider="foo", name="bar"; a, b);
}


