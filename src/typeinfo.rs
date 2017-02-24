use syntax::ast;
use rustc::ty;

use consts;


pub fn get_input_size(input_type: &ty::TypeVariants) -> i8 {
    //Given a type, provide a byte-size matching systemtap expectations
    //https://sourceware.org/systemtap/wiki/UserSpaceProbeImplementation
    //report an error if we cannot support it
    match *input_type {
        ty::TypeVariants::TyInt(int_type) => {
            match int_type {
                ast::IntTy::I8  => -1,
                ast::IntTy::I16 => -2,
                ast::IntTy::I32 => -4,
                ast::IntTy::I64 | ast::IntTy::Is => -(consts::POINTER_WIDTH_BYTES as i8),
                _ => panic!("Type unsupported by probe spec")
            }
        },
        ty::TypeVariants::TyUint(uint_type) => {
            match uint_type {
                ast::UintTy::U8  => 1,
                ast::UintTy::U16 => 2,
                ast::UintTy::U32 => 4,
                ast::UintTy::U64 | ast::UintTy::Us => consts::POINTER_WIDTH_BYTES as i8,
                _ => panic!("Type unsupported by probe spec")
            }
        },
        ty::TypeVariants::TyFloat(float_type) => {
            match float_type {
                ast::FloatTy::F32 => 4,
                ast::FloatTy::F64 => 8,
            }
        },
        ty::TyRef(_, _) | ty::TyRawPtr(ty::TypeAndMut {..}) => consts::POINTER_WIDTH_BYTES as i8,
        //&ty::TyAdt(_ /*std::ffi::OsString*/, _) => 8,
        //TyStr - ptr to str,
        //TySlice(ty),
        //TyAdt(adt_ref, substs)
        _ => {println!("bugme"); panic!("I don't know what to do with type: {:?}, report a bug.", input_type); }
   }
}

