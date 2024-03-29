#![no_main]
#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::spi::SpiBus;
use embedded_hal::digital::OutputPin;
use panic_halt as _;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rp2040_hal::entry]
fn main() -> ! {
    // The HAL has the PAC included
    let mut peripherals = rp2040_hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = rp2040_hal::Watchdog::new(peripherals.WATCHDOG);

    // Default clock configuration
    let clocks = rp2040_hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    )
    .unwrap();

    // Sets all pins to their default state
    let sio = rp2040_hal::Sio::new(peripherals.SIO);

    let pins = rp2040_hal::gpio::Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    // Configure as SPI sub
    let spi_sck = pins.gpio18.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio19.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio16.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let _spi_cs = pins.gpio17.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let spi = rp2040_hal::spi::Spi::<_, _, _, 8>::new(peripherals.SPI0, (spi_mosi, spi_miso, spi_sck));

    let mut spi = spi.init_slave(
        &mut peripherals.RESETS,
        embedded_hal::spi::MODE_3,
    );

    // Enable output on the GPIO pin, we'll send on it 5us long pulses with
    // 10us in between them. The number of pulses equals the value received
    // on the SPI bus.
    let mut gpio_pin = pins.gpio0.into_push_pull_output();

    // Read bytes as they arrive
    let mut timer = rp2040_hal::Timer::new(peripherals.TIMER, &mut peripherals.RESETS, &clocks);
    let mut read_buffer: [u8; 1] = [0];

    loop {
        if spi.read(&mut read_buffer).is_ok() {
            let _ = spi.flush();
        }
        for _ in 0..read_buffer[0] {
            let _ = gpio_pin.set_high();
            timer.delay_us(5);
            let _ = gpio_pin.set_low();
            timer.delay_us(10);
        }
    }
}
