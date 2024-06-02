#![no_main]
#![no_std]

use arrayref::array_ref;
use core::fmt::Write;
use embedded_hal::spi::SpiDevice;
use panic_halt as _;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::uart::{DataBits, StopBits, UartConfig};
use rp2040_hal::Clock;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

// Note that we're not interested in writing a BMP280 driver so
// we just try keep the amount of magic values to a minimum
const BMP280_ID_REG: u8 = 0xD0;
const BMP280_CHIP_ID: u8 = 0x58;
const BMP280_CALIB_DATA_ADDR: u8 = 0x88;
const BMP280_CTRL_MEAS_REG: u8 = 0xF4;
const BMP280_TEMP_REG: u8 = 0xFA;

// 32 bit fixed point temperature compensation algorithm taken from the BMP280 datasheet
fn compensate_temperature(adc_t: u32, dig_t1: u16, dig_t2: i16, dig_t3: i16) -> u32 {
    let var1: u32;
    let var2: u32;
    let t_fine: u32;

    var1 = (((adc_t >> 3) - ((dig_t1 as u32) << 1)) * (dig_t2 as u32)) >> 11;
    var2 = (((((adc_t >> 4) - (dig_t1 as u32)) * ((adc_t >> 4) - (dig_t1 as u32))) >> 12)
        * (dig_t3 as u32))
        >> 14;
    t_fine = var1 + var2;

    return (t_fine * 5 + 128) >> 8;
}

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

    // Configure as SPI main and let the embedded_hal_bus ExclusiveDevice handle the CS
    let spi_sck = pins.gpio18.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio19.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio16.into_function::<rp2040_hal::gpio::FunctionSpi>();
    let spi_cs = pins
        .gpio17
        .into_push_pull_output_in_state(rp2040_hal::gpio::PinState::High);

    let mut spi = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(
        rp2040_hal::spi::Spi::<_, _, _, 8>::new(peripherals.SPI0, (spi_mosi, spi_miso, spi_sck))
            .init(
                &mut peripherals.RESETS,
                clocks.peripheral_clock.freq(),
                8.kHz(),
                embedded_hal::spi::MODE_3,
            ),
        spi_cs,
    )
    .unwrap();

    // Read out device id as a way to check that power-on-reset is finished
    let mut read_buffer: [u8; 2] = [0x00; 2];
    while read_buffer[1] != BMP280_CHIP_ID {
        spi.transfer(&mut read_buffer, &[BMP280_ID_REG]).unwrap();
    }

    // Read out temperature compensation parameters (dig_t1, dig_t2, dig_t3 in the datasheet)
    let mut read_buffer: [u8; 7] = [0x00; 7];
    spi.transfer(&mut read_buffer, &[BMP280_CALIB_DATA_ADDR])
        .unwrap();

    let dig_t1: u16 = u16::from_le_bytes(*array_ref![read_buffer, 1, 2]);
    let dig_t2: i16 = i16::from_le_bytes(*array_ref![read_buffer, 3, 2]);
    let dig_t3: i16 = i16::from_le_bytes(*array_ref![read_buffer, 5, 2]);

    // Configure the device to measure temperature once
    // osrs_t osrs_p mode
    // 0 0 1  0 0 0  0 1
    spi.write(&[BMP280_CTRL_MEAS_REG & 0x7F, 0x21]).unwrap();

    // Read out the measurement and calculate temperature value according to the datasheet
    let mut read_buffer: [u8; 4] = [0x00; 4];
    spi.transfer(&mut read_buffer, &[BMP280_TEMP_REG]).unwrap();

    let adc_t = u32::from_be_bytes(read_buffer) >> 4;
    let temp = compensate_temperature(adc_t, dig_t1, dig_t2, dig_t3);

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
