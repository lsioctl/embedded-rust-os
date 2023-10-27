use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use lazy_static::lazy_static;

use pic8259::ChainedPics;
use spin;

// import from this lib crate
use crate::{println, print};
use crate::gdt;

// FFI
// Foreign calling convention
// for interrupt calling convention (different prologue ...)
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("Exception breakpoint, stack frame: {:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("Double fault triggered, stack frame: {:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_: InterruptStackFrame) {
    print!(".");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// we ne once more lazy_static to avoid mutable static
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        let double_fault_entry = idt.double_fault.set_handler_fn(double_fault_handler);
        // unsafe because we have to ensure the index is valid
        // and not already used for another exception
        unsafe {
            double_fault_entry.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        //idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);

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

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// the Mutex will allow us safe mutable access
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(
    // wrong offset would cause UB so unsafe
    unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    }
);

#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET
}

impl InterruptIndex {
    pub fn as_usize(self) -> usize {
        usize::from(self as u8)
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}