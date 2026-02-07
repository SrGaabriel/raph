#![no_std]
#![no_main]

use r_efi::efi;

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(
    image_handle: efi::Handle,
    system_table: *mut efi::SystemTable,
) -> efi::Status {
    let st = unsafe { &*system_table };
    let con_out = unsafe { &mut *st.con_out };

    let hello = [0x48u16, 0x65, 0x6C, 0x6C, 0x6F, 0x21, 0x0A, 0x00];
    unsafe {
        (con_out.output_string)(con_out, hello.as_ptr() as *mut efi::Char16);
    }

    efi::Status::SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
