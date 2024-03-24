#![no_main]
#![no_std]

use embedded_hal::pwm::SetDutyCycle;
use panic_halt as _;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rp2040_hal::entry]
fn main() -> ! {
    // The HAL has the PAC included
    let mut peripherals = rp2040_hal::pac::Peripherals::take().unwrap();
    let _core_peripherals = rp2040_hal::pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = rp2040_hal::Watchdog::new(peripherals.WATCHDOG);

    // Default clock configuration
    let _clocks = rp2040_hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Sets all pins to their default state
    let sio = rp2040_hal::Sio::new(peripherals.SIO);

    let pins = rp2040_hal::gpio::Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    // Init PWM slices
    let pwm_slices = rp2040_hal::pwm::Slices::new(peripherals.PWM, &mut peripherals.RESETS);

    // Set the period to 1000Hz (see chapter 4.5.2.6 in the datasheet)
    let mut pwm = pwm_slices.pwm0;
    pwm.set_top(62500);
    pwm.set_div_int(2);
    pwm.enable();

    // And the duty cycle to 25%
    let mut channel = pwm.channel_a;
    let _ = channel.set_duty_cycle(62500 / 4);
    channel.output_to(pins.gpio0);

    loop {}
}
