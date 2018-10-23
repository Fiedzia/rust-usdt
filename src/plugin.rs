use syntax::ast;
use syntax::ext::base::{ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::ext::hygiene::SyntaxContext;
use syntax::parse::token;
use syntax_pos::Span;
use syntax::ptr::P;
use syntax::symbol::Symbol;
use syntax::tokenstream::TokenTree;


use common::ProbeProperties;
use platform;

pub fn static_probe_expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
    -> Box<MacResult + 'static> {
/* 
    Syntax: 
        provider="provider", name="name" ; (expr type,)*
 
     Usage example:
         static_probe!(provider="", name="")
         static_probe!(provider="foo", name="bar"; some_variable f64, some_other_variable u64);

*/
    let mut probe_properties = ProbeProperties {
        provider: None,
        name: None,
        arguments: vec![],
    };
    let mut expressions: Vec<ast::Expr> = vec![];

    //parse comma-separated param=value pairs
    let mut idx = 0;
    if args.len() < 3 {
        panic!("not enough arguments");
    }
    while (args.len()-idx) >= 3 {
        match (&args[idx], &args[idx+1], &args[idx+2]) {
            (
                &TokenTree::Token(_, token::Token::Ident(ident, _)),
                &TokenTree::Token(_, token::Token::Eq),
                //TODO: allow static variables, maybe other types of Lit
                &TokenTree::Token(_, token::Token::Literal(token::Lit::Str_(s), _))
            ) => {
                    let ident_str = ident.name.to_string();
                    match ident_str.as_str() {
                        "provider" => { probe_properties.provider = Some(s.to_string()); },
                        "name" => { probe_properties.name = Some(s.to_string()); },
                        v => { panic!(format!("unexpected probe property: {}", v)); }
                    };
            },
            _ => { break; } //{ panic!(format!("unexpected: {:?}", args[idx])) }
        };
        idx += 3;
        if idx >= args.len() {
            break;
        }
        match args[idx] {
            TokenTree::Token(_, token::Token::Semi) => {idx += 1; break},
            TokenTree::Token(_, token::Token::Comma) => {idx += 1},
            TokenTree::Token(_, token::Token::Ident(_, _)) => {},
            _ => break
        }
    }
    if probe_properties.provider.is_none() {
        probe_properties.provider = Some("".to_string());
    }

    if probe_properties.name.is_none() {
        panic!("name is required")
    }


    //parse expression list
    if idx < args.len() {
        let remainder = args.windows(args.len() - idx).last().unwrap();
        let mut parser = cx.new_parser_from_tts(remainder);
        loop {
            let expr  = &*parser.parse_expr().unwrap();
            println!("expr={:?}", expr);
            parser.eat(&token::Comma);
            let _type = &*parser.parse_ty().unwrap();
            probe_properties.arguments.push((expr.clone(), _type.clone()));
            parser.eat(&token::Comma);
            if parser.eat(&token::Eof) {
                break
            } else {
                println!("rem: {:?}", remainder);
            }
        }

    }
    
    let asm_text = platform::implementation::generate_asm_code(&probe_properties).unwrap_or("".to_string());
    println!("asm:{}", asm_text);
    let asm_expressions: Vec<(Symbol, P<ast::Expr>)> = expressions.into_iter().map(|expr| {(Symbol::intern("nor"), P(expr))}).collect();
    let stmt = ast::Stmt{
        id: ast::DUMMY_NODE_ID,
        node: ast::StmtKind::Expr(P(ast::Expr{
            id: ast::DUMMY_NODE_ID,
            span: sp,
            attrs: vec![].into(),
            node: ast::ExprKind::InlineAsm(P(ast::InlineAsm{
                asm: Symbol::intern(&asm_text),
                asm_str_style: ast::StrStyle::Cooked,
                outputs: vec![],
                inputs: asm_expressions,
                clobbers: vec![],
                volatile: true,
                alignstack: false,
                dialect: ast::AsmDialect::Att,
                ctxt: SyntaxContext::empty(),
            }))
        })),
        span: sp
    };
    let block = P(ast::Block {
       stmts: vec![stmt],
       id: ast::DUMMY_NODE_ID,
       rules: ast::BlockCheckMode::Unsafe(ast::UnsafeSource::UserProvided),
       span: sp,
       recovered: false,
    });
    MacEager::expr(cx.expr_block(block))
}

