use sbi_rt::{self, Physical};
use memory_addr::VirtAddr;

use crate::mem::virt_to_phys;

/// The maximum number of bytes that can be read at once.
const MAX_RW_SIZE: usize = 256;

/// Writes a single byte to the console using legacy SBI interface.
///
/// This is a fallback when newer SBI interfaces are unavailable.
#[allow(deprecated)]
fn fallback_putchar(c: u8) {
    sbi_rt::legacy::console_putchar(c as usize);
}

/// Writes a single byte to the console.
/// Uses new SBI interface if available.
pub fn putchar(c: u8) {
    // Try using modern interface first
    #[allow(deprecated)]
    if sbi_rt::console_write_byte(c).value == 0 {
        // Fallback to legacy if failed
        fallback_putchar(c);
    }
}

/// Tries to write a slice of bytes to the console using the modern SBI interface.
///
/// Returns the number of bytes successfully written.
fn try_write_bytes(bytes: &[u8]) -> usize {
    let pa = virt_to_phys(VirtAddr::from_ptr_of(bytes.as_ptr())).as_usize();

    sbi_rt::console_write(Physical::new(
        bytes.len().min(MAX_RW_SIZE),
        pa,
        0,
    ))
    .value
}

/// Writes a slice of bytes to the console.
/// Falls back to legacy interface if the modern interface is not available.
pub fn write_bytes(bytes: &[u8]) {
    let mut offset = 0;
    while offset < bytes.len() {
        let len = try_write_bytes(&bytes[offset..]);
        if len == 0 {
            // fallback: legacy interface
            for &b in &bytes[offset..] {
                fallback_putchar(b);
            }
            break;
        }
        offset += len;
    }
}

/// Reads bytes from the console into the given mutable buffer.
/// Returns the number of bytes read.
///
/// This only works on systems that support `sbi_console_read`.
pub fn read_bytes(buf: &mut [u8]) -> usize {
    let pa = virt_to_phys(VirtAddr::from_mut_ptr_of(buf.as_mut_ptr())).as_usize();
    sbi_rt::console_read(Physical::new(buf.len().min(MAX_RW_SIZE), pa, 0)).value
}
