#![no_main]
#![no_std]

// This implementation takes things one step further by using the PAC
//
// Note that the PAC is accessed through the HAL only because I couldn't
// fix the critical-section dependency of the rp2040-pac crate
//
// The delay implementation is using the cortex_m crates for convenience, it
// could of course be written in a similar style

use panic_halt as _;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const TEST_GPIO_PIN: usize = 0;

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = rp2040_hal::pac::Peripherals::take().unwrap();

    // Enable the PADS_BANK0 and IO_BANK0 peripherals
    let resets = &peripherals.RESETS;
    resets
        .reset()
        .modify(|_, w| w
            .pads_bank0().clear_bit()
            .io_bank0().clear_bit()
        );

    // Wait for them to reset
    loop {
        let resets_reset_done = resets.reset_done().read();

        if resets_reset_done.pads_bank0().bit_is_set() &&
            resets_reset_done.io_bank0().bit_is_set() {
            break;
        }
    }

    // Make sure that the GPIO pin is initially output disabled in SIO
    // and that it is low
    let sio = &peripherals.SIO;

    sio.gpio_oe_clr().write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });
    sio.gpio_out_clr().write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });

    // Disable pull-down, disable input, make sure that output can be enabled
    let gpio_pad = &peripherals.PADS_BANK0.gpio(TEST_GPIO_PIN);
    gpio_pad.write(|w| w
        .pde().clear_bit()
        .ie().clear_bit()
        .od().clear_bit()
    );

    // Configure the GPIO pin to be controlled through SIO
    let gpio_pin = &peripherals.IO_BANK0.gpio(TEST_GPIO_PIN);
    gpio_pin.gpio_ctrl().write(|w| w
        .funcsel().sio()
    );

    // Enable output on the GPIO pin
    sio.gpio_oe_set().write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });

    // Set up a delay using the cortex_m crate
    let core = rp2040_hal::pac::CorePeripherals::take().unwrap();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, 6000000);

    loop {
        // Set the GPIO pin high
        sio.gpio_out_set().write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });
        delay.delay_ms(1000);
        // Set the GPIO pin low
        sio.gpio_out_clr().write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });
        delay.delay_ms(1000);
    }
}
