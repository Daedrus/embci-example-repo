#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const TEST_GPIO_PIN: usize = 0;

#[entry]
fn main() -> ! {
    let peripherals = rp2040_pac::Peripherals::take().unwrap();
    let core = rp2040_pac::CorePeripherals::take().unwrap();

    let resets = &peripherals.RESETS;

    resets
        .reset
        .modify(|_, w| w
            .pads_bank0().clear_bit()
            .io_bank0().clear_bit()
        );

    loop {
        let resets_done = resets.reset_done.read();

        if resets_done.pads_bank0().bit_is_set() &&
            resets_done.io_bank0().bit_is_set() {
            break;
        }
    }

    let sio = &peripherals.SIO;

    sio.gpio_oe_clr.write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });
    sio.gpio_out_clr.write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });

    let led_gpio_pad = &peripherals.PADS_BANK0.gpio[TEST_GPIO_PIN];
    led_gpio_pad.write(|w| w
        .pde().clear_bit()
        .ie().clear_bit()
        .od().clear_bit()
    );

    let led_gpio_pin = &peripherals.IO_BANK0.gpio[TEST_GPIO_PIN];
    led_gpio_pin.gpio_ctrl.write(|w| w
        .funcsel().sio_0()
    );

    sio.gpio_oe_set.write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });

    let mut delay = cortex_m::delay::Delay::new(core.SYST, 6000000);

    loop {
        sio.gpio_out_set.write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });
        delay.delay_ms(1000);
        sio.gpio_out_clr.write(|w| unsafe { w.bits(1 << TEST_GPIO_PIN) });
        delay.delay_ms(1000);
    }
}
