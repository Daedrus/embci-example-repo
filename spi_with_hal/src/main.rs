#![no_main]
#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::spi::SpiBus;
use embedded_hal::digital::OutputPin;
use panic_halt as _;
use rp2040_hal::Clock;

use rp2040_hal::fugit::RateExtU32;

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

    // Configure as SPI main
    let spi_sck = pins.gpio2.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio3.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio4.into_function::<rp2040_hal::gpio::FunctionSpi>();

    // We have to "manually" control the CS pin
    let mut spi_cs = pins.gpio5.into_push_pull_output();
    let _ = spi_cs.set_high();

    let spi = rp2040_hal::spi::Spi::<_, _, _, 8>::new(peripherals.SPI0, (spi_mosi, spi_miso, spi_sck));

    // MODE 3 has easy CS logic (no need to pulse it HIGH)
    let mut spi = spi.init(
        &mut peripherals.RESETS,
        clocks.peripheral_clock.freq(),
        8.kHz(),
        embedded_hal::spi::MODE_3,
    );

    // Send a byte every 500ms
    let mut timer = rp2040_hal::Timer::new(peripherals.TIMER, &mut peripherals.RESETS, &clocks);
    let mut write_buffer: [u8; 1] = [1];

    let _ = spi.flush();

    // Wait some time so that the sub can start up
    timer.delay_ms(500);

    loop {
        let _ = spi_cs.set_low();
        if spi.write(&mut write_buffer).is_ok() {
        }
        let _ = spi.flush();
        let _ = spi_cs.set_high();
        timer.delay_ms(500);
        write_buffer[0] += 1;
    }
}
