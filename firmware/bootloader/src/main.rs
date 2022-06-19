#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception};

#[cfg(feature = "defmt")]
use defmt_rtt as _;

use embassy_boot_nrf::*;
use embassy_nrf::nvmc::Nvmc;

#[entry]
fn main() -> ! {
    let p = embassy_nrf::init(Default::default());
    let mut bl = BootLoader::default();
    let start = bl.prepare(&mut SingleFlashProvider::new(&mut WatchdogFlash::start(
        Nvmc::new(p.NVMC),
        p.WDT,
        10,
    )));
    unsafe { bl.load(start) }
}

#[no_mangle]
#[cfg_attr(target_os = "none", link_section = ".HardFault.user")]
unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn DefaultHandler(_: i16) -> ! {
    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;
    let irqn = core::ptr::read_volatile(SCB_ICSR) as u8 as i16 - 16;

    panic!("DefaultHandler #{:?}", irqn);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf();
}
