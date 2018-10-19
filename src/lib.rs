#[macro_use]
extern crate libasm;

// High level overview:
//
//TODO:
//
// wrap input params in ( as i64) (or whatever is appropriate)
//
//

//macro to obtain type of a variable
//taken from https://stackoverflow.com/questions/21747136/how-do-i-print-the-type-of-a-variable-in-rust
//written by phicr
pub trait TypeInfo {
    fn type_name() -> String;
    fn type_of(&self) -> String;
}
    
macro_rules! impl_type_info {
    ($($name:ident$(<$($T:ident),+>)*),*) => {
        $(impl_type_info_single!($name$(<$($T),*>)*);)*
    };
}
    
macro_rules! mut_if {
    ($name:ident = $value:expr, $($any:expr)+) => (let mut $name = $value;);
    ($name:ident = $value:expr,) => (let $name = $value;);
}
    
macro_rules! impl_type_info_single {
    ($name:ident$(<$($T:ident),+>)*) => {
        impl$(<$($T: TypeInfo),*>)* TypeInfo for $name$(<$($T),*>)* {
            fn type_name() -> String {
                mut_if!(res = String::from(stringify!($name)), $($($T)*)*);
                $(
                    res.push('<');
                    $(
                        res.push_str(&$T::type_name());
                        res.push(',');
                    )*
                    res.pop();
                    res.push('>');
                )*
                res
            }
            fn type_of(&self) -> String {
                $name$(::<$($T),*>)*::type_name()
            }
        }
    }
}
    
impl<'a, T: TypeInfo + ?Sized> TypeInfo for &'a T {
    fn type_name() -> String {
        let mut res = String::from("&");
        res.push_str(&T::type_name());
        res
    }
    fn type_of(&self) -> String {
        <&T>::type_name()
    }
}

macro_rules! type_of {
    ($x:expr) => { (&$x).type_of() };
}


/// static_probe! macro
/// Usage: static_probe!(provider="foo", name="bar");
///        let foo = 1u32;
///        static_probe!(provider="foo", name="bar", foo);
///

macro_rules! static_probe {
    (provider = $probe_provider :expr, name= $probe_name :expr  $(, $var:expr )* ) => (
    println!("provider: {} name: {}", $probe_provider, $probe_name);
    $(println!("x:{}", $x);)*
    );
}




