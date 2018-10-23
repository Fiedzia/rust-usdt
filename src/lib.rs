// High level overview:
//
//  Here are the steps we do here:
//
//  1. register compiler plugin providing static_probe macro.
//     This will convert probe macros into
//     unsafe { asm!("..."); } blocks,
//     inserting MAGIC_ASM_MARK and attribute values into asm text.
//     Here we also connect expression passed to probe to asm inputs.
//     On this level we are working with token stream.
//
//  2. Run MIR pass to find asm invocations we generated previously.
//     and discover types of all expressions that were passed to macro.
//
//  3. Run another MIR pass modifying asm text to final value.
//
//  We need to do last two steps separately due to running them with different
//  MIR mutability.
//
//TODO:
//
// wrap input params in ( as i64) (or whatever is appropriate)
//
//

#![feature(quote, plugin_registrar, rustc_private)]


extern crate syntax;
extern crate syntax_pos;
extern crate rustc;
extern crate rustc_data_structures;
extern crate rustc_mir;
extern crate rustc_plugin;

use rustc_plugin::Registry;

mod common;
mod consts;
mod platform;
mod plugin;
mod typeinfo;

#[plugin_registrar]
pub fn registrar(reg: &mut Registry) {
    reg.register_macro("static_probe", plugin::static_probe_expand);
}
