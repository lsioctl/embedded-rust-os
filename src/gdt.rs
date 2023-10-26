use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};

use lazy_static::lazy_static;

/// The GDT is a legacy which was used for memory segmentation
/// before paging became a standard
/// it was used in older architecture to isolate programs from
/// each other.
/// Segmentation is not longer supported in 64 bit mode
/// but GDT is still used mostly for:
/// * switching between kernel and user space
/// * loading a TSS structure 

/// TSS (Task state segment) is also a legacy structure
/// it was historicaly used for holding for example processor
/// register state about a task in 32 bit mode for context switching
/// Hardware context switching is not supported in 64 bit mode
/// and the TSS has mostly changed
/// Now it holds two stack tables:
/// * Privilege Stack table [u64; 3]
/// * IST [u64; 7]
/// and I/O Map base address (this is the only common point for 32/64 bits modes)


// any index would do the job in fact
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            // we don't have memory management yet
            // so we can't allocate properly a stack
            // we use an unsafe static mut array in the mean time
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            // we write the top address, as the stacks on x86 grows downwards
            // note that we don't have a page guard so we
            // should do fancy stuff in our double fault handler
            // or we could corrupt
            // the memory below the stack
            stack_end
        };
        tss
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors {code_selector, tss_selector})
    };
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT.0.load();

    // could break memory safety with invalid selectors
    unsafe {
        // reload the code segment register
        CS::set_reg(GDT.1.code_selector);
        // load the TSS
        load_tss(GDT.1.tss_selector);
    }
}