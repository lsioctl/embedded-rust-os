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

mod vga_buffer;

use core::panic::PanicInfo;

use vga_buffer::print_something;

/// This function is called on panic.
/// with no_std we have to implement our own
#[panic_handler]
// should never return so marked as diverging function
// with the return type ! (never)
// PanicInfo containes the file and the line where the
// panic happened, and optional message
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


static HELLO: &[u8] = b"Hello World!";

// overwriting the crt0 entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // raw pointer to the VGA buffer address
    // let vga_buffer = 0xb8000 as *mut u8;

    // HELLO.iter().enumerate().for_each(|(i, &byte)| {
    //     // This is not how it should be handled in Rust
    //     // as we could write before or after the VGA buffer
    //     unsafe {
    //         // ASCII byte
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         // color (background and font) byte
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0x0c;
    //     }
    // });

    print_something();
    loop {}
}