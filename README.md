# rust-usdt
Inject USDT probes into rust code

Based on work done by Josh Stone [https://github.com/cuviper/rust-libprobe]

Usage:

Cargo.toml:
```toml
[dependencies.rust-usdt]
git = "https://github.com/Fiedzia/rust-usdt"
```

in src/main.rs (as example, but you can insert probes in any place in your code)
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

compile:
```sh
cargo build
```

Make sure probes were generated:
```sh
readelf -n ./target/debug/demo | grep NT_STAPSDT -A 4
  stapsdt              0x00000048       NT_STAPSDT (SystemTap probe descriptors)
    Provider: foo
    Name: bar
    Location: 0x000000000000741b, Base: 0x000000000003dc21, Semaphore: 0x0000000000000000
    Arguments: 8@-281(%rbp) 8@-289(%rbp)
```

Run bcc trace to trace them:
```sh
sudo /usr/share/bcc/tools/trace 'u:/home/maciej/git/rust-usdt/demo/demo1/target/debug/usdt_demo:bar "%d", arg1' 

PID    TID    COMM         FUNC             -
8163   8163   demo         bar              0
```
(you will need to run your app in separate terminal window to see the results)


For more details, see [documentation](doc/doc.md).
