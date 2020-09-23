use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

fn kernel_data_segment() -> Descriptor {
	use x86_64::structures::gdt::DescriptorFlags as Flags;

	let flags = Flags::USER_SEGMENT | Flags::PRESENT | Flags::WRITABLE | Flags::LONG_MODE;
    Descriptor::UserSegment(flags.bits())
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(kernel_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
				tss_selector,
            },
        )
    };
}

struct Selectors {
	code_selector: SegmentSelector,
	data_selector: SegmentSelector,
	tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{load_ds, load_es, load_ss, set_cs};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_ds(GDT.1.data_selector);
        load_es(GDT.1.data_selector);
        load_ss(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}
