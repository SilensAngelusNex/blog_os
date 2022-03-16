#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use blog_os::{exit_qemu, gdt, serial_print, serial_println, QemuExitCode};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    blog_os::test_panic_handler(info);
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success)
}

fn init() {
    blog_os::gdt::init();
    TEST_IDT.load();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::overflow_stack...\t");
    init();

    recurse_forever();
    unreachable!("Shouldn't be able to continue after overflow.")
}

#[allow(unconditional_recursion)]
fn recurse_forever() {
    recurse_forever();

    // Prevents tail recursion optimizations
    volatile::Volatile::new(0).read();
}
