#[cfg(target_pointer_width = "32")]
pub const POINTER_WIDTH_BYTES: u8 = 4;

#[cfg(target_pointer_width = "64")]
pub const POINTER_WIDTH_BYTES: u8 = 8;
