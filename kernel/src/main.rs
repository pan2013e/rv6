#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::empty_loop)]

use core::panic::PanicInfo;
use kernel::{logging, trap};

#[no_mangle]
pub extern "C" fn os_main() -> ! {
    #[cfg(test)]
    test_main();

    trap::init();
    logging::init();
    log::info!("Hello, RV6!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log::error!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        log::error!("Panicked: {}", info.message().unwrap());
    }
    kernel::sbi::shutdown()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}