use syntax::ast::{Ty, TyKind};


///Given a type, provide a byte-size matching systemtap expectations
///https://sourceware.org/systemtap/wiki/UserSpaceProbeImplementation
///report an error if we cannot support it
pub fn get_input_size(input_type: &Ty) -> i8 {
    match input_type.node {
        TyKind::Path(None, ref p) => {
            let path_str = format!("{}", p);
            match path_str.as_ref() {
                "u8" => 1,
                "i8" => -1,
                "u16" => 2,
                "i16" => -2,
                "u32" => 4,
                "i32" => -4,
                "u64" => 8,
                "i64" => -8,
                "f32" => 4,
                "f64" => 8,
                _ => 8
            }
        },
        _ => 8
    }
}
