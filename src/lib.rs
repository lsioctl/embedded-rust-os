#![no_std]
#![feature(abi_x86_interrupt)]

pub mod vga_buffer;
mod interrupts;

pub fn init() {
    interrupts::init_idt();
}
