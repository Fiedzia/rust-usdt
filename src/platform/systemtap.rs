use rustc::{self, mir};
use rustc::ty::Ty;

use consts;
use common::ProbeProperties;
use typeinfo::get_input_size;



pub fn generate_asm_code(_: &rustc::hir::InlineAsm,
                     _ : &[mir::Operand], //inputs
                     input_types: &[Ty],
                     probe_properties: ProbeProperties) -> String {

    let mut arg_str: String = "".to_string();
    for (idx, input) in input_types.iter().enumerate() {
        println!("sty:{:?}", &input.sty);
        let input_size = get_input_size(&input.sty);
        let s = match idx {
            0 => format!("{input_size}@${idx}", idx=idx, input_size=input_size),
            _ => format!(" {input_size}@${idx}", idx=idx, input_size=input_size),
        };
        arg_str.push_str(&s);
    }
    let asm_code = format!(r##"
        #probeasm
        990:    nop
                .pushsection .note.stapsdt,"?","note"
                .balign 4
                .4byte 992f-991f, 994f-993f, 3
        991:    .asciz "stapsdt"
        992:    .balign 4
        993:    .{bw}byte 990b
                .{bw}byte _.stapsdt.base
                .{bw}byte 0 // FIXME set semaphore address
                .asciz "{provider}"
                .asciz "{name}"
                .asciz "{arg_str}"
        994:    .balign 4
                .popsection
        .ifndef _.stapsdt.base
                .pushsection .stapsdt.base,"aG","progbits",.stapsdt.base,comdat
                .weak _.stapsdt.base
                .hidden _.stapsdt.base
        _.stapsdt.base: .space 1
                .size _.stapsdt.base, 1
                .popsection
        .endif
    "##,
    bw=consts::POINTER_WIDTH_BYTES,
    arg_str=arg_str,
    provider=probe_properties.provider.unwrap(),
    name=probe_properties.name.unwrap()
    );
    asm_code
}
