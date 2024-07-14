#![no_main]
#![no_std]

use arrayref::array_ref;
use core::fmt::Write;
use embedded_hal::i2c::I2c;
use panic_halt as _;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::uart::{DataBits, StopBits, UartConfig};
use rp2040_hal::Clock;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

// Note that we're not interested in writing a TMP117 driver so
// we just try keep the amount of magic values to a minimum
const TMP117_TEMP_RESULT_REG: u8 = 0x00;
const TMP117_CONFIGURATION_REG: u8 = 0x01;
const TMP117_DEVICE_ID_REG: u8 = 0x0F;
const TMP117_I2C_ADDRESS: u8 = 0x48;
const TMP117_RESOLUTION: f32 = 0.0078125;

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

    // Configure I2C
    let sda_pin: rp2040_hal::gpio::Pin<_, rp2040_hal::gpio::FunctionI2C, _> = pins.gpio20.reconfigure();
    let scl_pin: rp2040_hal::gpio::Pin<_, rp2040_hal::gpio::FunctionI2C, _> = pins.gpio21.reconfigure();

    let mut i2c = rp2040_hal::I2C::i2c0(
        peripherals.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut peripherals.RESETS,
        &clocks.system_clock,
    );

    // Read out device id
    let mut read_buffer: [u8; 2] = [0x00; 2];
    let _ = i2c.write_read(TMP117_I2C_ADDRESS, &[TMP117_DEVICE_ID_REG], &mut read_buffer);

    // Wait for temperature data to be ready
    let mut read_buffer: [u8; 2] = [0x00; 2];
    while read_buffer[0] & 0x20 == 0 {
        let _ = i2c.write_read(TMP117_I2C_ADDRESS, &[TMP117_CONFIGURATION_REG], &mut read_buffer);
    }

    // Read out temperature
    let mut read_buffer: [u8; 2] = [0x00; 2];
    let _ = i2c.write_read(TMP117_I2C_ADDRESS, &[TMP117_TEMP_RESULT_REG], &mut read_buffer);

    let temp: f32 = f32::from(u16::from_be_bytes(*array_ref![read_buffer, 0, 2])) * TMP117_RESOLUTION;

    // Configure UART and write the temperature result on it
    let uart_pins = (
        pins.gpio0.into_function::<rp2040_hal::gpio::FunctionUart>(),
        pins.gpio1.into_function::<rp2040_hal::gpio::FunctionUart>(),
    );

    let mut uart = rp2040_hal::uart::UartPeripheral::new(
        peripherals.UART0,
        uart_pins,
        &mut peripherals.RESETS,
    )
    .enable(
        UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
        clocks.peripheral_clock.freq(),
    )
    .unwrap();

    writeln!(uart, "{temp}\r").unwrap();

    loop {}
}
