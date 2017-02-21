use std::marker::PhantomData;

use syntax::ast;
use syntax::ext::base::{ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use syntax_pos::{Span, NO_EXPANSION};
use syntax::ptr::P;
use syntax::symbol::Symbol;
use syntax::tokenstream::TokenTree;

use rustc::{hir, mir};
use rustc::mir::{BasicBlock, Location, Mir, Statement};
use rustc::mir::visit::{Visitor as MirVisitor, MutVisitor};
use rustc::mir::transform::{MirPass, MirSource, Pass};
use rustc_plugin::Registry;
use rustc::ty::{self, Ty, TyCtxt};

use common::ProbeProperties;
use plugin::ProbeMirPlugin;
use typeinfo::get_input_size;



pub fn generate_asm_code(_: &mut rustc::hir::InlineAsm,
                     _ : &Vec<mir::Operand>, //inputs
                     input_types: &[Ty],
                     probe_properties: ProbeProperties) -> String {
    String::from("")
}
