#![no_std]

use core::panic::PanicInfo;

#[global_allocator]
static ALLOCATOR: common::alloc::Allocator = common::alloc::Allocator::new();

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    api::abort(info)
}
