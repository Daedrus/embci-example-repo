#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    loop {
    }
}
