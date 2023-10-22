#![no_main]
#![no_std]

// This implementation is how one would write embedded Rust code in "C style"
//
// The delay implementation is using the cortex_m crates for convenience, it
// could of course be written in a similar style

use panic_halt as _;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const TEST_GPIO_PIN: u32 = 0;

// Definitions like these are what you would typically see in C code
//
// Of course, one would use structs and bitfields but the purpose here
// is not to provide a good implementation but rather an idea of how
// Rust code can directly read and write from/to registers
//
// Constructs such as these _are_ used in the PAC but they are neatly
// embellished with careful typing and other Rust features

const RESETS_BASE: u32 = 0x4000_c000;
const RESETS_RESET_OFFSET: u32 = 0x00;
const RESETS_RESET_DONE_OFFSET: u32 = 0x08;
const RESETS_RESET_PADS_BANK0_MASK: u32 = 1 << 8;
const RESETS_RESET_IO_BANK0_MASK: u32 = 1 << 5;

const SIO_BASE: u32 = 0xd000_0000;
const SIO_GPIO_OUT_SET_OFFSET: u32 = 0x14;
const SIO_GPIO_OUT_CLR_OFFSET: u32 = 0x18;
const SIO_GPIO_OE_SET_OFFSET: u32 = 0x24;
const SIO_GPIO_OE_CLR_OFFSET: u32 = 0x28;

const PADS_BANK0_BASE: u32 = 0x4001_c000;
const PADS_BANK0_GPIOX_PDE_MASK: u32 = 1 << 2;
const PADS_BANK0_GPIOX_IE_MASK: u32 = 1 << 6;
const PADS_BANK0_GPIOX_OD_MASK: u32 = 1 << 7;

const IO_BANK0_BASE: u32 = 0x4001_4000;
const IO_BANK0_GPIOX_CTRL_OFFSET: u32 = 0x04;
const IO_BANK0_GPIOX_CTRL_FUNCSEL_SHIFT: u32 = 0x0;
const IO_BANK0_GPIOX_CTRL_FUNCSEL_SIO: u32 = 5;

#[cortex_m_rt::entry]
fn main() -> ! {
    unsafe {
        // Enable the PADS_BANK0 and IO_BANK0 peripherals
        let resets_reset = (RESETS_BASE + RESETS_RESET_OFFSET) as *mut u32;
        *resets_reset &= !(RESETS_RESET_PADS_BANK0_MASK | RESETS_RESET_IO_BANK0_MASK);

        // Wait for them to reset
        loop {
            let resets_reset_done = (RESETS_BASE + RESETS_RESET_DONE_OFFSET) as *const u32;

            if *resets_reset_done & (RESETS_RESET_PADS_BANK0_MASK | RESETS_RESET_IO_BANK0_MASK)
                == (RESETS_RESET_PADS_BANK0_MASK | RESETS_RESET_IO_BANK0_MASK)
            {
                break;
            }
        }

        // Make sure that the GPIO pin is initially output disabled in SIO
        // and that it is low
        let sio_gpio_oe_set = (SIO_BASE + SIO_GPIO_OE_SET_OFFSET) as *mut u32;
        let sio_gpio_oe_clr = (SIO_BASE + SIO_GPIO_OE_CLR_OFFSET) as *mut u32;
        let sio_gpio_out_set = (SIO_BASE + SIO_GPIO_OUT_SET_OFFSET) as *mut u32;
        let sio_gpio_out_clr = (SIO_BASE + SIO_GPIO_OUT_CLR_OFFSET) as *mut u32;

        *sio_gpio_oe_clr = 1 << TEST_GPIO_PIN;
        *sio_gpio_out_clr = 1 << TEST_GPIO_PIN;

        // Disable pull-down, disable input, make sure that output can be enabled
        let gpio_pad = (PADS_BANK0_BASE + (1 + TEST_GPIO_PIN) * 0x04) as *mut u32;
        *gpio_pad =
            !(PADS_BANK0_GPIOX_PDE_MASK | PADS_BANK0_GPIOX_IE_MASK | PADS_BANK0_GPIOX_OD_MASK);

        // Configure the GPIO pin to be controlled through SIO
        let gpio_pin_ctrl =
            (IO_BANK0_BASE + TEST_GPIO_PIN * 0x04 + IO_BANK0_GPIOX_CTRL_OFFSET) as *mut u32;
        *gpio_pin_ctrl = IO_BANK0_GPIOX_CTRL_FUNCSEL_SIO << IO_BANK0_GPIOX_CTRL_FUNCSEL_SHIFT;

        // Enable output on the GPIO pin
        *sio_gpio_oe_set = 1 << TEST_GPIO_PIN;

        // Set up a delay using the cortex_m crate
        let core = cortex_m::Peripherals::take().unwrap();
        let mut delay = cortex_m::delay::Delay::new(core.SYST, 6000000);

        loop {
            // Set the GPIO pin high
            *sio_gpio_out_set = 1 << TEST_GPIO_PIN;
            delay.delay_ms(1000);
            // Set the GPIO pin low
            *sio_gpio_out_clr = 1 << TEST_GPIO_PIN;
            delay.delay_ms(1000);
        }
    }
}
