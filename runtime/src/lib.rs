#![no_std]

use core::panic::PanicInfo;

#[global_allocator]
static ALLOCATOR: raph_common::alloc::Allocator = raph_common::alloc::Allocator::new();

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    raph_api::abort(info)
}
