#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]

use blog_os::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    bad_assert();
    serial_println!("[test did not panic]");
    blog_os::exit_qemu(QemuExitCode::Failure);
}

fn bad_assert() {
    serial_print!("should_panic::bad_assert...\t");
    assert_eq!(0, 1);
}
