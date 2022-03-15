#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(not(test))]
    {
        blog_os::panic_print!("{}\n", info);
        loop {}
    }

    #[cfg(test)]
    blog_os::test_panic_handler(info);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    main();

    loop {}
}

fn main() {
    println!("Hello World{}", "!");
}
