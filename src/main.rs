#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello, World!";
const VGA_BUFFER: *mut u8 = 0xb8000 as _;

#[no_mangle]
pub extern "C" fn _start() ->! {
    for (i, &byte) in HELLO.iter().enumerate() {
        let char_start =  i as isize * 2;
        unsafe {
            *VGA_BUFFER.offset(char_start) = byte;
            *VGA_BUFFER.offset(char_start + 1) = 0xb;
        }
    }
    
    loop {}
}