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

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!(
        "Level 4 page table at: {:?}",
        level_4_page_table.start_address()
    );

    let ptr = 0x2055d3 as *mut u64;

    let x;
    unsafe {
        x = *ptr;
    }
    println!("Read {:#x}.", x);

    unsafe {
        *ptr = 42;
    }
    println!("Write worked.");
}
