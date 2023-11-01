#![no_std]
// we don't want to use the normal entry point chain
// normally for rust linked with std lib
// * execution starts in a C runtime library (crt0)
// which prepares a C application (setting the stack,
// arguments in the right register)
// * then crt0 invoke entry point of the Rust runtime
// marked by the `start` language item
// Rust runtime is very minimal (stack overflow guards,
// printing backtrace on panic, ...)
// * Rust runtime then calls the main function
// note than when overwriting crt0 entry point (_start function)
// the linker will complain because it will assume that we
// are running on C runtime
// So we will have to had a custom target triple with none as the os
// like: thumbv7em-none-eabihf
#![no_main]

use core::panic::PanicInfo;

// import from our lib crate
use embedded_rust_os::{println, print};

//use crate::println;

/// This function is called on panic.
/// with no_std we have to implement our own
#[panic_handler]
// should never return so marked as diverging function
// with the return type ! (never)
// PanicInfo containes the file and the line where the
// panic happened, and optional message
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// #[allow(unconditional_recursion)]
// fn stack_overflow() {
//     stack_overflow();
// }

// overwriting the crt0 entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("{} is the response to {}", 42, "anything");

    embedded_rust_os::init();

    // send the int3 (breakpoint, #BP) exception
    //x86_64::instructions::interrupts::int3();

    // once interrupt is handled, execution continue

    //println!("Still alive");

    // trigger a page fault
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    //stack_overflow();
    loop {
        //print!("_");
    }
}