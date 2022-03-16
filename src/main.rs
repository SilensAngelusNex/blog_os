#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::{hlt_loop, println};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(not(test))]
    {
        blog_os::panic_print!("{}\n", info);
        hlt_loop();
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

    hlt_loop();
}

fn main() {
    println!("Hello World{}", "!");
    blog_os::init();

    // unsafe {
    //     // SAFETY: Uh, no.
    //     (0xdeadbeef as *mut u64).write_volatile(42);
    // }

    println!("Look ma, no crash!");
}
