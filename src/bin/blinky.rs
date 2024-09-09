#![no_std]
#![no_main]

use cortex_m_rt::entry;
use gd32f4::gd32f425::{Gpioc, Pmu, Rcu};
// use gd32f4::gd32f425::rcu:
use core::arch::asm;
use libm::sinf;
use panic_probe as _;

// Implement this C macro in rust
// #define RCU_MODIFY      {volatile uint32_t i; \
//                         RCU_CFG0 |= RCU_AHB_CKSYS_DIV2; \
//                         for(i=0;i<50000;i++); \
//                         RCU_CFG0 |= RCU_AHB_CKSYS_DIV4; \
//                         for(i=0;i<50000;i++);}
// macro_rules! RCU_MODIFY {
//     () => {{
//         let mut i: u32;
//         unsafe {
//             pac::Rcu::steal()
//                 .cfg0()
//                 .modify(|_, w| w.ahb_cksys_div2().set_bit());
//         }
//         for i in 0..50000 {}
//         unsafe {
//             pac::Rcu::steal()
//                 .cfg0()
//                 .modify(|_, w| w.ahb_cksys_div4().set_bit());
//         }
//         for i in 0..50000 {}
//     }};
// }

// use `main` as the entry point of this application
// `main` is not allowed to return
#[entry]
fn main() -> ! {
    // initialization
    system_init();

    // Set up our peripherals
    setup_peripherals();

    // let mut time: f32 = 0.0;
    let mut ctr: u32 = 0;
    // let mut sin: f32 = 0.0;

    // Toggle the LED
    let gpio_c = unsafe { Gpioc::steal() };

    loop {
        // Set the LEDs high
        gpio_c.octl().modify(|_, w| w.octl1().set_bit());
        gpio_c.octl().modify(|_, w| w.octl2().set_bit());

        // Delay for a bit
        for _ in 0..500000 {
            unsafe {
                asm!("nop");
            }
        }

        // Set the LEDs low
        gpio_c.octl().modify(|_, w| w.octl1().clear_bit());
        gpio_c.octl().modify(|_, w| w.octl2().clear_bit());

        // More delay
        for _ in 0..500000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}

/// Set up our peripherals.
fn setup_peripherals() {
    let rcu = unsafe { Rcu::steal() };

    // Enable the GPIOA peripheral
    rcu.ahb1en().modify(|_, w| w.paen().set_bit());
    // Enable the GPIOB peripheral
    rcu.ahb1en().modify(|_, w| w.pben().set_bit());
    // Enable the GPIOC peripheral
    rcu.ahb1en().modify(|_, w| w.pcen().set_bit());

    // Configure GPIOC pins 1 and 2 as LED outputs
    let gpio_c = unsafe { Gpioc::steal() };
    gpio_c.ctl().modify(|_, w| unsafe { w.ctl1().bits(0b01) });
    gpio_c.ctl().modify(|_, w| unsafe { w.ctl2().bits(0b01) });

    // Set outputs speed to max
    gpio_c.ospd().modify(|_, w| unsafe { w.ospd1().bits(0b11) });
    gpio_c.ospd().modify(|_, w| unsafe { w.ospd2().bits(0b11) });

    // Set outputs low to start
    gpio_c.octl().modify(|_, w| w.octl1().clear_bit());
    gpio_c.octl().modify(|_, w| w.octl2().clear_bit());
}

/// The system init function tranlsated from system_gd32f4xx.c
/// from the GD32F4xx firmware library. All magic numbers are
/// taken from the GD32F4xx firmware library.
fn system_init() {
    // Create a handle to the RCU peripheral
    let rcu = unsafe { Rcu::steal() };
    // Create a handle to the PMU peripheral
    let pmu = unsafe { Pmu::steal() };

    // Set the IRC16MEN bit to enable the internal 16 MHz RC oscillator
    rcu.ctl0().modify(|_, w| w.irc16men().set_bit());

    rcu.cfg0()
        .modify(|_, w| unsafe { w.ahbpsc().bits(0b1000) });

    // Spin for 50000 cycles
    for _ in 0..50000 {
        unsafe {
            asm!("nop");
        }
    }

    rcu.cfg0()
        .modify(|_, w| unsafe { w.ahbpsc().bits(0b1001) });

    // Spin for 50000 cycles
    for _ in 0..50000 {
        unsafe {
            asm!("nop");
        }
    }

    // Clear the SCSS bits
    rcu.cfg0().modify(|_, w| unsafe { w.scs().bits(0b00) });

    /* Reset HXTALEN, CKMEN and PLLEN bits */
    rcu.ctl0().modify(|_, w| {
        w.hxtalen()
            .clear_bit()
            .ckmen()
            .clear_bit()
            .pllen()
            .clear_bit()
    });

    /* Reset HSEBYP bit */
    rcu.ctl0().modify(|_, w| w.hxtalbps().clear_bit());

    /* Reset CFG0 register */
    rcu.cfg0().modify(|_, w| unsafe { w.bits(0x00000000) });

    // Wait until IRC16M is selected as system clock
    while rcu.cfg0().read().scs().bits() != 0b00 {}

    /* Reset PLLCFGR register */
    rcu.pll().modify(|_, w| unsafe { w.bits(0x24003010) });

    /* Disable all interrupts */
    // RCU_INT = 0x00000000U;
    rcu.int().write(|w| unsafe { w.bits(0x00000000) });

    // Configure RCU for 8MHz external clock and 200MHz clock speed
    // Enable HXTAL
    rcu.ctl0().modify(|_, w| w.hxtalen().set_bit());

    /* wait until HXTAL is stable or the startup time is longer than HXTAL_STARTUP_TIMEOUT */
    let mut timeout: u32 = 0;
    let mut stab_flag: u32 = 0;
    loop {
        timeout += 1;
        stab_flag = rcu.ctl0().read().hxtalstb().bit() as u32;
        if !((stab_flag == 0) && (timeout != 0xFFFF)) {
            break;
        }
    }

    // If that failed
    if rcu.ctl0().read().hxtalstb().bit_is_clear() {
        panic!("HXTAL failed to start");
    }

    // ENABLE APB1 PMU
    rcu.apb1en().modify(|_, w| w.pmuen().set_bit());
    // Set PMU to LDO VS
    pmu.ctl().modify(|_, w| unsafe { w.ldovs().bits(0b11) });

    // Now the crystal is stable, we can use it as the system clock
    // Set the AHB prescaler to 1
    rcu.cfg0().modify(|_, w| unsafe { w.ahbpsc().bits(0b000) });
    // Set APB2 to AHB/2
    rcu.cfg0().modify(|_, w| unsafe { w.apb2psc().bits(0b100) });
    // Set APB1 to AHB/4
    rcu.cfg0().modify(|_, w| unsafe { w.apb1psc().bits(0b101) });

    /* Configure the main PLL, PSC = 8, PLL_N = 400, PLL_P = 2, PLL_Q = 9 */
    //    RCU_PLL = (8U | (400U << 6U) | (((2U >> 1U) - 1U) << 16U) |
    //               (RCU_PLLSRC_HXTAL) | (9U << 24U));
    rcu.pll().modify(|_, w| unsafe {
        w.pllpsc()
            .bits(8)
            .plln()
            .bits(400)
            .pllp()
            .bits(2)
            .pllq()
            .bits(9)
            .pllsel()
            .set_bit()
    });

    // Enable PLL
    rcu.ctl0().modify(|_, w| w.pllen().set_bit());

    // Wait until PLL is stable
    while rcu.ctl0().read().pllstb().bit_is_clear() {}

    // Enable the high-drive to extend to clock frequency to 200MHz
    pmu.ctl().modify(|_, w| w.hden().set_bit());
    // Wait until the high-drive is stable
    while pmu.cs().read().hdrf().bit_is_clear() {}
    // Select high-drive mode
    pmu.ctl().modify(|_, w| w.hds().set_bit());
    // Wait until the high-drive is stable
    while pmu.cs().read().hdsrf().bit_is_clear() {}

    // Set PLL as system clock
    rcu.cfg0().modify(|_, w| unsafe {w.scs().bits(0b10) });

    // Wait until PLL is selected as the system clock
    while rcu.cfg0().read().scss().bits() == 0b10 {}
}
