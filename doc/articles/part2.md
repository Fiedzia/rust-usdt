# Rust bcc/BPF function tracing

This is second part of series of articles about using tracing with Rust. See the first (here: Tracing - your println 2.0)

Inspired by Brendan Gregg article [Golang bcc/BPF Function Tracing](http://www.brendangregg.com/blog/2017-01-31/golang-bcc-bpf-function-tracing.html) about using bpf tracing with golang, I explored how it works with Rust.

This is going to be very technical article. I assume you are familiar with tracing, bpf and [bcc tools](https://github.com/iovisor/bcc).
I am using stable rust, Ubuntu 16.10 and latest bcc-tools.

Let start with the basics:

```sh
cargo new --bin --name demo demo
cd demo
```

lets put this src/main.rs
```rust
fn main() {
    println!("Hello, BPF");
}
```
and run:
```sh
cargo build
```
this way we created our binary, located in target/debug/demo
The first example in Brendan's article is this one:
```sh
funccount 'go:fmt.*'
```
... and this won't work here. Unlike gcgo, rust stdlib is not exposed as a shared object,  but compiled statically into every program.
```sh
ldd target/debug/demo  
       linux-vdso.so.1 =&gt;  (0x00007ffc15d35000) 
       libdl.so.2 =&gt; /lib/x86_64-linux-gnu/libdl.so.2 (0x00007f8a338f9000) 
       librt.so.1 =&gt; /lib/x86_64-linux-gnu/librt.so.1 (0x00007f8a336f1000) 
       libpthread.so.0 =&gt; /lib/x86_64-linux-gnu/libpthread.so.0 (0x00007f8a334d3000) 
       libgcc_s.so.1 =&gt; /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007f8a332bc000) 
       libc.so.6 =&gt; /lib/x86_64-linux-gnu/libc.so.6 (0x00007f8a32ef5000) 
       /lib64/ld-linux-x86-64.so.2 (0x000055f2e8edb000)
```
as you can see, there is no such thing like "librust" here.
To find functions used in our program, we have look at the program itself:
(note: Rust equivalent of go `fmt.Println` function is `core::fmt::write`)
```sh
objdump -t | grep write

0000000000007b80 l     F .text  0000000000000153  _ZN72_$LT$std..io..buffered..BufWriter$LT$W$GT$$u20$as$u20$std..io..Write$GT$5write17h0106b162517f717eE 
0000000000009190 l     F .text  00000000000000c6  _ZN94_$LT$std..io..Write..write_fmt..Adaptor$LT$$u27$a$C$$u20$T$GT$$u20$as$u20$core..fmt..Write$GT$9write_str17h60e0256dabd7fb9fE
```

You will see a lot of names like that. The reason for that is that naming of exported function is more complex in Rust than it is in Go, and therefore compiler must resort to name mangling to ensure there are no conflicts. You can undo it to some extend if you ask objdump to demangle names:
```sh
objdump -t --demangle | grep write
0000000000008bb0 l     F .text  000000000000025c std::io::Write::write_all::h304ace53756db31d 
0000000000008e10 l     F .text  0000000000000162 std::io::Write::write_all::h7c10a51290981adc
```
This form is easier to read, but to use bcc we will need to give it mangled form. We may not be sure which function we should look like, so let's try pass all of them to funccount. To do that, we will need two terminals: one to run following command:
```sh
sudo /usr/share/bcc/tools/funccount '/tmp/demo/target/debug/demo:*write*' 
Tracing 35 functions for "/tmp/demo/target/debug/demo:*write*"... Hit Ctrl-C to end. 
```
and another one to run our application, so that we have something to trace. We will do that for all following examples. Once you do that, go back to first terminal and you'll see which functions were calld and how many times:

```
FUNC                                    COUNT 
_ZN3std2io5Write9write_all17h304ace53756db31dE        1 
_ZN94_$LT$std..io..Write..write_fmt..Adaptor$LT$$u27$a$C$$u20$T$GT$$u20$as$u20$core..fmt..Write$GT$9write_str17h60e0256dabd7fb9fE        1 
_ZN75_$LT$std..io..stdio..StdoutLock$LT$$u27$a$GT$$u20$as$u20$std..io..Write$GT$5write17h2ed843efb0ae4a51E        1 
_ZN57_$LT$std..io..stdio..Stdout$u20$as$u20$std..io..Write$GT$9write_fmt17hc45e966c4ed23748E        1 
_ZN4core3fmt5write17ha410d2e3733df97bE        1 
_ZN72_$LT$std..io..buffered..BufWriter$LT$W$GT$$u20$as$u20$std..io..Write$GT$5write17h0106b162517f717eE        2
```
Mangled names are not nice to read, but you can see its working. Rust stdlib is fairly complex thing though, lets try write our own function to make things simpler:
```rust
fn add(a: u64, b: u64) -> u64 {
    a+b
}

fn main(){
    println!(add(42, 13));
}
```
```sh
cargo build 
````
first, lets find how our function is actually called:
```sh
objdump -t --demangle ./target/debug/demo  |  grep add
    00000000000056b0 l     F .text  000000000000004a              demo::add::hfc45e365eaaa12e0
```
that's demangled name, the reals one is:
```
objdump -t  ./target/debug/demo  |  grep hfc45e365eaaa12e0
    00000000000056b0 l     F .text  000000000000004a              _ZN4demo3add17hfc45e365eaaa12e0E
```
(if its not clear, look at first column with address to match them). Knowing exact name, we can trace it:
```sh
/usr/share/bcc/tools/trace './target/debug/demo:_ZN4demo3add17hfc45e365eaaa12e0E'

PID    TID    COMM         FUNC              
20483  20483  demo         _ZN4demo3add17hfc45e365eaaa12e0E
```
So again, its ugly, but its working. I believe bcc does not support demangling rust names (and if does, it doesn't have a way to pass : character as part of function name, since this character separates parts of probe names).
However, we can ask rust not to mangle names of our functions:
```rust
#[no_mangle]
pub fn add(a: u64, b: u64) -> u64 {
    a+b
}
```
```sh
objdump -t  ./target/debug/demo  |  grep add
    00000000000056b0 g     F .text  000000000000004a              add
```

so lets trace it again, and this time we will also print arguments:
```sh
/usr/share/bcc/tools/trace 'target/debug/demo:add "%d %d", arg1, arg2'

PID    TID    COMM         FUNC             -
21217  21217  demo         add              42 13
```
Unlike Go, Rust plays with standard conventions, so no changes to bcc were required. All values are printed as expected. Unless you run in release mode:
```sh
cargo build --release

/usr/share/bcc/tools/trace '/tmp/demo/target/release/demo:add "%d %d", arg1, arg2' 
could not determine address of symbol add
```
What's happened? Our add function is so simple that compiler decided it will be faster just to insert addition directly into code that called it. This optimisation is called inlining, and is not applied in debug mode to make debugging easier. For the purpose of this article, we can disable it:

```rust
#[inline(never)]
#[no_mangle]
pub fn add(a: u64, b: u64) -> u64 {
    a+b
}
```
and now trace will work as expected.

Summary:

Using bcc directly on rust code is possible, but its not always most pleasant experience. There are some tricks we can use to make it easier - and bcc may gain better support for Rust in the future, however there are better ways to improve instrumentation of our code, amd I will discuss them in my next article.
