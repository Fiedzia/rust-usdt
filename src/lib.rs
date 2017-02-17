/*
 * TODO:
 *  wrap input params in ( as i64(or whatever is appropriate))
 *
 * */


#![feature(quote, plugin_registrar, rustc_private)]

extern crate syntax;
extern crate syntax_pos;
extern crate rustc;
extern crate rustc_plugin;

use std::marker::PhantomData;

use syntax::ast;
use syntax::ext::base::{ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::parse::{filemap_to_tts, ParseSess, token};
use syntax_pos::{Span, NO_EXPANSION};
use syntax::ptr::P;
use syntax::symbol::Symbol;
use syntax::tokenstream::{TokenTree, TokenStream};

use rustc::{hir, mir};
use rustc::mir::{BasicBlock, Location, Mir, Statement};
use rustc::mir::visit::{Visitor as MirVisitor, MutVisitor};
use rustc::mir::transform::{MirPass, MirSource, Pass};
use rustc_plugin::Registry;
use rustc::ty::{Ty, TyCtxt};


//use this to mark inline asm we generate
//so that we do not modify unrelated asm code
static MAGIC_ASM_MARK: &'static str = "#probeasm";

//mir visitor used for read-only pass
//to retrieve input type information
struct ProbeVisitor<'a, 'tcx: 'a> { 
    mir: &'a mir::Mir<'tcx>,
    input_types: Vec<Ty<'tcx>>
}

impl <'a, 'tcx>ProbeVisitor<'a, 'tcx> {

    fn set_input_types_from_asm(&mut self, asm_inputs: &Vec<mir::Operand>){
        //inspect asm inputs and set self.input_types to match them
        for input in asm_inputs {
			match input {
				&mir::Operand::Consume(ref lv) => {
					match lv {
						&mir::Lvalue::Static(d) => panic!("bug"),
						&mir::Lvalue::Local(def_id) => {
                            self.input_types.push(self.mir.local_decls[def_id].ty);
                        },
						_ => { panic!("bug")}
					};
				},
				&mir::Operand::Constant(ref c) => panic!("bug")
			};
        }
        assert!(asm_inputs.len() == self.input_types.len());
    }
}

impl<'a, 'tcx> MirVisitor<'tcx> for ProbeVisitor<'a, 'tcx> {

    fn visit_statement(&mut self,
        _: BasicBlock,
        statement: &Statement<'tcx>,
        _: Location) {
        if let mir::StatementKind::Assign(_, ref rval) = statement.kind {
            if let &mir::Rvalue::InlineAsm{ref asm, ref inputs, outputs: _} = rval {
                if !is_probe_asm(&asm) {
                    return
                };
                self.set_input_types_from_asm(&inputs);
            }
        }
    }
}


pub fn is_probe_asm(asm: &hir::InlineAsm) -> bool {
    //is that that our asm?
    let asm_str = asm.asm.as_str();
    (asm_str.len() >= MAGIC_ASM_MARK.len() && &asm_str[0..MAGIC_ASM_MARK.len()] == MAGIC_ASM_MARK)
}

//mutable mir visitor
struct MutProbeVisitor<'a, 'tcx: 'a> {
    input_types: Vec<Ty<'tcx>>,
    phantom: PhantomData<&'a ()>
}


impl <'a, 'tcx> MutVisitor<'tcx> for MutProbeVisitor<'a, 'tcx> {
    fn visit_statement(&mut self,
        _: BasicBlock,
        statement: &mut Statement<'tcx>,
        _: Location) {
        if let mir::StatementKind::Assign(ref mut lval, ref mut rval) = statement.kind {

            if let &mut mir::Rvalue::InlineAsm{ref mut asm, ref mut inputs, outputs: _} = rval {

                if !is_probe_asm(asm) {
                    return
                };
                let asm_str = asm.asm.as_str();

                for (idx, input) in inputs.iter_mut().enumerate() {

                    //println!("input: {:?}", input);
                    println!("input: {:?} type: {:?}", input, self.input_types[idx]);
                }
	    		asm.asm = Symbol::intern("NOP");	
			}
		}

	}

}


impl <'a, 'tcx> Pass for MutProbeVisitor<'a, 'tcx> {}


struct ProbeMirPlugin {}

impl <'tcx> Pass for ProbeMirPlugin {}
impl <'tcx> MirPass<'tcx> for ProbeMirPlugin {

    fn run_pass<'a>(&mut self, types: TyCtxt<'a, 'tcx, 'tcx>, _: MirSource, mir: &mut Mir<'tcx>) {
        let input_types = {
			let mut pv = ProbeVisitor {mir: &mir, input_types: vec![]};
			pv.visit_mir(&mir);
            pv.input_types
        };
		let mut mvp = MutProbeVisitor{phantom: PhantomData, input_types: input_types};
		mvp.visit_mir(mir);
    }
}

#[plugin_registrar]
pub fn registrar(reg: &mut Registry) {

    let mut visitor = ProbeMirPlugin {};
    reg.register_mir_pass(Box::new(visitor));
    reg.register_macro("static_probe", static_probe_expand);
}


pub fn string_to_tts(source_str: String) -> Vec<TokenTree> {
    let ps = ParseSess::new();
    filemap_to_tts(&ps, ps.codemap().new_filemap("bogofile".to_string(), None, source_str))
}


fn string_to_ts(string: &str) -> TokenStream {
    string_to_tts(string.to_owned()).into_iter().collect()
}


fn static_probe_expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
/* 
    Syntax: 
        provider="provider", name="name" (expr;)*
 
     Usage example:
         static_probe!(provider="", name="")
         static_probe!(provider="foo",  name="bar"; baz; baz.field )

*/
        let mut provider: Option<String> = None;
        let mut name: Option<String> = None;
        let mut expressions: Vec<ast::Expr> = vec![];
		let total = 0;

        //parse comma-separated param=value pairs
        let mut idx = 0;
        if args.len() < 3 {
            panic!("not enough arguments");
        }
        while (args.len()-idx) >= 3 {
            match (&args[idx], &args[idx+1], &args[idx+2]) {
                (
                    &TokenTree::Token(_, token::Token::Ident(ident)),
                    &TokenTree::Token(_, token::Token::Eq),
                    &TokenTree::Token(_, token::Token::Literal(lit, ast_name))
                ) => {
                        let ident_str = ident.name.as_str().clone().to_string();
                        match ident_str.as_str() {
                            "provider" => { provider = Some(ident_str); },
                            "name" => { name = Some(ident_str); },
                            _ => { panic!("unexpected value"); }
                        };
                },
                _ => { panic!("unexpected") }
            };
            idx += 3;
            if idx >= args.len() {
                break;
            }
            match args[idx] {
                TokenTree::Token(_, token::Token::Semi) => {idx += 1; break},
                TokenTree::Token(_, token::Token::Comma) => {idx += 1},
                TokenTree::Token(_, token::Token::Ident(_)) => {},
                _ => break
            }
        }
        if provider.is_none() {
            let provider = Some("".to_string());
        }

        if name.is_none() {
            panic!("name is required")
        }


        //parse expression list
        if idx < args.len() {
            let remainder = args.windows(args.len() - idx).last().unwrap();
            let mut parser = cx.new_parser_from_tts(remainder);
            while true {
                let expr  = parser.parse_expr();
                if expr.is_ok() {
                    expressions.push(expr.unwrap().unwrap().clone());
                } else {
                    panic!("err")
                }
                if parser.check(&token::Comma) {
                    parser.bump();
                }
                if parser.check(&token::Eof) {
                    break
                }
            }

        }
        
        let asm_text = format!("#probeasm\n#provider={provider}\n#name={name}\n#", provider=provider.unwrap(), name=name.unwrap());
        
        let asm_expressions: Vec<(Symbol, P<ast::Expr>)> = expressions.into_iter().map(|expr| {(Symbol::intern("nor"), P(expr))}).collect();
        let stmt = ast::Stmt{
            id: ast::DUMMY_NODE_ID,
            node: ast::StmtKind::Expr(P(ast::Expr{
                id: ast::DUMMY_NODE_ID,
                span: sp,
                attrs: ast::ThinVec::new(),
                node: ast::ExprKind::InlineAsm(P(ast::InlineAsm{
                    asm: Symbol::intern(&asm_text),
                    asm_str_style: ast::StrStyle::Cooked,
                    outputs: vec![],
                    inputs: asm_expressions,
                    clobbers: vec![],
                    volatile: true,
                    alignstack: false,
                    dialect: ast::AsmDialect::Intel,
                    expn_id: NO_EXPANSION 
                }))
            })),
            span: sp
        };
        let block = P(ast::Block {
           stmts: vec![stmt],
           id: ast::DUMMY_NODE_ID,
           rules: ast::BlockCheckMode::Unsafe(ast::UnsafeSource::UserProvided),
           span: sp,
        });
        MacEager::expr(cx.expr_block(block))
}





