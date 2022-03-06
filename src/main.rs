#![no_std]
#![no_main]

use core::panic::PanicInfo;
use blog_os::panic_print;
#[cfg(not(test))]
use blog_os::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_print!("{}\n", info);
    loop {}
}

#[no_mangle]
pub extern "Rust" fn _main() {
    #[cfg(test)]
    blog_os::exit_qemu(blog_os::QemuExitCode::Success);

    #[cfg(not(test))]
    main();
}

#[cfg(not(test))]
fn main() {
    println!("Hello, {}!", "World");
}