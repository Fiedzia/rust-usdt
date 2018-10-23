//TODO: make it feature-selected instead of os based


#[cfg(target_os = "linux")]
mod systemtap;

#[cfg(target_os = "linux")]
pub mod implementation {
    pub use platform::systemtap::generate_asm_code;

}

#[cfg(not(target_os = "linux"))]
mod dummy;

#[cfg(not(target_os = "linux"))]
pub mod implementation {
    pub use platform::dummy::generate_asm_code;
}
