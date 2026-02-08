use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{define_interrupt_handler, idt::set_handler};

pub fn init_pic() {
    unsafe {
        core::arch::asm!(
            // ICW1: initialize, expect ICW4
            "mov al, 0x11",
            "out 0x20, al", // master command port
            "out 0xA0, al", // slave command port
            // ICW2: vector offset
            "mov al, 0x20",
            "out 0x21, al", // master: IRQ0-7 → vectors 32-39
            "mov al, 0x28",
            "out 0xA1, al", // slave: IRQ8-15 → vectors 40-47
            // ICW3: cascade wiring
            "mov al, 0x04",
            "out 0x21, al", // master: slave is on IRQ2 (bit 2 set)
            "mov al, 0x02",
            "out 0xA1, al", // slave: cascade identity is 2
            // ICW4: 8086 mode
            "mov al, 0x01",
            "out 0x21, al",
            "out 0xA1, al",
            // mask: only IRQ1 (keyboard) unmasked on master, all masked on slave
            "mov al, 0xFD",
            "out 0x21, al",
            "mov al, 0xFF",
            "out 0xA1, al",
        );
    }
}

pub fn register_interrupt_handlers() {
    unsafe { set_handler(33, keyboard_isr as u64) };
}

define_interrupt_handler!(keyboard_isr, keyboard_handler);

static mut KEY_BUFFER: [u8; 256] = [0; 256];
static WRITE_POS: AtomicUsize = AtomicUsize::new(0);
static READ_POS: AtomicUsize = AtomicUsize::new(0);

pub fn push_scancode(scancode: u8) {
    let w = WRITE_POS.load(Ordering::Relaxed);
    unsafe { KEY_BUFFER[w & 0xFF] = scancode };
    WRITE_POS.store(w.wrapping_add(1), Ordering::Release);
}

pub fn pop_scancode() -> Option<u8> {
    let r = READ_POS.load(Ordering::Relaxed);
    let w = WRITE_POS.load(Ordering::Acquire);
    if r == w {
        return None;
    }
    let code = unsafe { KEY_BUFFER[r & 0xFF] };
    READ_POS.store(r.wrapping_add(1), Ordering::Relaxed);
    Some(code)
}

extern "C" fn keyboard_handler() {
    let scancode: u8;
    unsafe { core::arch::asm!("in al, 0x60", out("al") scancode) };
    unsafe { core::arch::asm!("mov al, 0x20", "out 0x20, al") };
    push_scancode(scancode);
}
