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
use rustc_mir::transform::{MirPass, MirSource};
use rustc::ty::{Ty, TyCtxt};

use rustc_data_structures::thin_vec::ThinVec;


use common::ProbeProperties;
use platform;


//use this to mark inline asm we generate
//so that we do not modify unrelated asm code
static MAGIC_ASM_MARK: &'static str = "#probeasm";




//mir visitor used for read-only pass
//to retrieve input type information
pub struct ProbeVisitor<'a, 'tcx: 'a> { 
    mir: &'a mir::Mir<'tcx>,
    input_types: Vec<Ty<'tcx>>
}

impl <'a, 'tcx>ProbeVisitor<'a, 'tcx> {

    fn set_input_types_from_asm(&mut self, asm_inputs: &[mir::Operand]){
        //inspect asm inputs and set self.input_types to match them
        for input in asm_inputs {
            match *input {
                mir::Operand::Consume(ref lv) => {
                    match *lv {
                        mir::Place::Static(_) => panic!("bug"),
                        mir::Place::Local(def_id) => {
                            self.input_types.push(self.mir.local_decls[def_id].ty);
                        },
                        _ => { panic!("bug")}
                    };
                },
                mir::Operand::Constant(_) => panic!("bug")
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
        if let mir::StatementKind::InlineAsm{ref asm, ref inputs, ..} = statement.kind {
            if !is_probe_asm(asm) {
                return
            };
            self.set_input_types_from_asm(inputs);
        }
    }
}


pub fn is_probe_asm(asm: &hir::InlineAsm) -> bool {
    //is that that our asm?
    let asm_str = asm.asm.as_str();
    (asm_str.len() >= MAGIC_ASM_MARK.len() && &asm_str[0..MAGIC_ASM_MARK.len()] == MAGIC_ASM_MARK)
}

//mutable mir visitor
pub struct MutProbeVisitor<'a, 'tcx: 'a> {
    input_types: Vec<Ty<'tcx>>,
    phantom: PhantomData<&'a ()>
}


impl <'a, 'tcx> MutVisitor<'tcx> for MutProbeVisitor<'a, 'tcx> {

    fn visit_statement(&mut self,
        _: BasicBlock,
        statement: &mut Statement<'tcx>,
        _: Location) {

        if let mir::StatementKind::InlineAsm{ref mut asm, ref mut inputs, ..} = statement.kind {

            if !is_probe_asm(asm) {
                return
            };
            let mut probe_properties = ProbeProperties{name: None, provider: None};
            for line in asm.asm.to_string().as_str().lines() {
                if line.contains('=') {
                    let k_v:Vec<&str> = line.splitn(2, '=').collect();
                    //skip first character (#)
                    let k:String = k_v[0].chars().skip(1).collect();
                    match k.as_ref() {
                        "name" => probe_properties.name = Some(k_v[1].to_string()),
                        "provider" => probe_properties.provider = Some(k_v[1].to_string()),
                        _ => panic!("unknown attribute")
                    };
                }
            }
            assert!(probe_properties.name.is_some(), "missing probe name");
            assert!(probe_properties.provider.is_some(), "missing probe provider");
            for (idx, input) in inputs.iter_mut().enumerate() {

                println!("input: {:?} type: {:?}", input, self.input_types[idx]);
            }

            asm.asm = Symbol::intern(&platform::implementation::generate_asm_code(asm, inputs, &self.input_types, probe_properties));
        }

    }

}


//impl <'a, 'tcx> Pass for MutProbeVisitor<'a, 'tcx> {}


pub struct ProbeMirPlugin {}

//impl <'tcx> Pass for ProbeMirPlugin {}
impl MirPass for ProbeMirPlugin {

    fn run_pass<'a, 'tcx>(&self, _: TyCtxt<'a, 'tcx, 'tcx>, _: MirSource, mir: &mut Mir<'tcx>) {
        let input_types = {
            let mut pv = ProbeVisitor {mir: mir, input_types: vec![]};
            pv.visit_mir(mir);
            pv.input_types
        };
        let mut mvp = MutProbeVisitor{phantom: PhantomData, input_types: input_types};
        mvp.visit_mir(mir);
    }
}


pub fn static_probe_expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
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
                    //TODO: allow static variables, maybe other types of Lit
                    &TokenTree::Token(_, token::Token::Literal(token::Lit::Str_(s), _))
                ) => {
                        let ident_str = ident.name.to_string();
                        match ident_str.as_str() {
                            "provider" => { provider = Some(s.to_string()); },
                            "name" => { name = Some(s.to_string()); },
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
            provider = Some("".to_string());
        }

        if name.is_none() {
            panic!("name is required")
        }


        //parse expression list
        if idx < args.len() {
            let remainder = args.windows(args.len() - idx).last().unwrap();
            let mut parser = cx.new_parser_from_tts(remainder);
            loop {
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
                attrs: ThinVec::new(),
                node: ast::ExprKind::InlineAsm(P(ast::InlineAsm{
                    asm: Symbol::intern(&asm_text),
                    asm_str_style: ast::StrStyle::Cooked,
                    outputs: vec![],
                    inputs: asm_expressions,
                    clobbers: vec![],
                    volatile: true,
                    alignstack: false,
                    dialect: ast::AsmDialect::Att,
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

