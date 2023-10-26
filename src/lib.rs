#![no_std]
#![feature(abi_x86_interrupt)]

pub mod vga_buffer;
mod interrupts;
mod gdt;

pub fn init() {
    interrupts::init_idt();
    gdt::init();
}
