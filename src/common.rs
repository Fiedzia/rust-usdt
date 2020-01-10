use syntax::ast::{Expr, Ty};

pub struct ProbeProperties {
    pub provider: Option<String>,
    pub name: Option<String>,
    pub arguments: Vec<(Expr, Ty)>,
}
