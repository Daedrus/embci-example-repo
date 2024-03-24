#![no_main]
#![no_std]

use panic_halt as _;

use embedded_hal::digital::OutputPin;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[rp2040_hal::entry]
fn main() -> ! {
    // The HAL has the PAC included
    let mut peripherals = rp2040_hal::pac::Peripherals::take().unwrap();

    // Sets all pins to their default state
    let sio = rp2040_hal::Sio::new(peripherals.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    // Enable output on the GPIO pin
    let mut gpio_pin = pins.gpio0.into_push_pull_output();

    // Set up a delay using the cortex_m crate
    let core = rp2040_hal::pac::CorePeripherals::take().unwrap();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, 6000000);

    loop {
        let _ = gpio_pin.set_high();
        delay.delay_ms(1000);
        let _ = gpio_pin.set_low();
        delay.delay_ms(1000);
    }
}
