#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    blog_os::exit_qemu(blog_os::QemuExitCode::Success);
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
