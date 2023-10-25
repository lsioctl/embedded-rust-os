use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use lazy_static::lazy_static;

// import from this lib crate
use crate::println;

// FFI
// Foreign calling convention
// for interrupt calling convention (different prologue ...)
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("Exception breakpoint, stack frame: {:#?}", stack_frame);
}

// we ne once more lazy_static to avoid mutable static
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        idt
    };
}

pub fn init_idt() {
    // load the IDT in the cpu with lidt
    // loads needs a &'static self
    // as the cpu will access this table
    // for every interrupts
    // until another IDT is loaded
    IDT.load();  
}

