#![no_std]
#![no_main]

use core::ptr::addr_of;

extern crate alloc;
extern crate common;
extern crate runtime;

#[repr(C)]
pub struct MemoryMapInfo {
    pub entries: *const u8,
    pub entry_count: usize,
    pub entry_size: usize,
}

#[repr(C)]
pub struct FramebufferInfo {
    pub base: u64,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixel_format: u32,
}

#[repr(C)]
pub struct BootInfo {
    pub memory_map: MemoryMapInfo,
    pub framebuffer: FramebufferInfo,
}

#[repr(C, packed)]
struct GdtEntry(u64);

impl GdtEntry {
    const fn null() -> Self {
        GdtEntry(0)
    }

    const fn kernel_code() -> Self {
        GdtEntry(0x00AF9A000000FFFF)
    }

    const fn kernel_data() -> Self {
        GdtEntry(0x00CF92000000FFFF)
    }
}

#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}

static GDT: [GdtEntry; 3] = [
    GdtEntry::null(),
    GdtEntry::kernel_code(),
    GdtEntry::kernel_data(),
];

const KERNEL_CS: u16 = 0x08;
const KERNEL_DS: u16 = 0x10;

unsafe fn init_gdt() {
    let descriptor = GdtDescriptor {
        limit: (core::mem::size_of_val(&GDT) - 1) as u16,
        base: GDT.as_ptr() as u64,
    };

    unsafe {
        core::arch::asm!(
            "lgdt [{}]",
            "push {}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov ds, {ds:x}",
            "mov es, {ds:x}",
            "mov fs, {ds:x}",
            "mov gs, {ds:x}",
            "mov ss, {ds:x}",
            in(reg) &descriptor,
            in(reg) KERNEL_CS as u64,
            ds = in(reg) KERNEL_DS as u64,
            tmp = lateout(reg) _,
        )
    };
}

unsafe fn init_idt() {
    let descriptor = IdtDescriptor {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: addr_of!(IDT) as u64,
    };

    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &descriptor,
        )
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    unsafe {
        init_gdt();
        init_idt();
    };

    let info = unsafe { &*boot_info };
    let fb = info.framebuffer.base as *mut u32;
    let pixels = (info.framebuffer.stride * info.framebuffer.height) as usize;
    for i in 0..pixels {
        unsafe {
            *fb.add(i) = 0x0000FF00;
        }
    }

    loop {}
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attrs: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    const fn new(offset: u64, selector: u16, ist: u8, type_attrs: u8) -> Self {
        IdtEntry {
            offset_low: offset as u16,
            selector,
            ist,
            type_attrs,
            offset_mid: (offset >> 16) as u16,
            offset_high: (offset >> 32) as u32,
            zero: 0,
        }
    }

    const fn missing() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attrs: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::missing(); 256];
