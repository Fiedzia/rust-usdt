# rust-usdt
Inject USDT probes into rust code

Based on work done by Josh Stone [https://github.com/cuviper/rust-libprobe]

Usage:

```rust
#![feature(asm)]
#![feature(plugin)]
#![plugin(rust_usdt)]


fn main() {
    let a = 0i64;
	let b = 1i64;
    static_probe!(provider="foo", name="bar"; a, b);

}
```

This won't insert probes yet - I'm still working on asm code to do it.
It will print information about data types of expressions passes to macro,
which will allow to use this information when generating probe asm code.
