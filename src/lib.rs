#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{convert::From, fmt::Debug};

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    hlt_loop();
}

pub fn init() {
    gdt::init();
    interrupts::init_itd();
    unsafe {
        interrupts::init_pics();
    }
    x86_64::instructions::interrupts::enable();
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn test_runner(tests: &[&dyn Testable]) -> ! {
    init();

    println!("Running {} tests", tests.len());
    for test in tests {
        test.run()
    }

    exit_qemu(QemuExitCode::Success);
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T: Fn() -> ()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<Self>());
        self();
        serial_println!("[ok]")
    }
}

impl From<()> for QemuExitCode {
    fn from(_: ()) -> Self {
        Self::Success
    }
}

impl<T: Into<QemuExitCode>> From<Option<T>> for QemuExitCode {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(inner) => inner.into(),
            None => Self::Failure,
        }
    }
}

impl<T: Into<QemuExitCode>, E: Debug> From<Result<T, E>> for QemuExitCode {
    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(inner) => inner.into(),
            Err(_) => {
                // TODO: Print the error? Not sure where to print it.
                Self::Failure
            }
        }
    }
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
