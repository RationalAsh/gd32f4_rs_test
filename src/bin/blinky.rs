#![no_std]
#![no_main]

use cortex_m_rt::entry;
use gd32f4::gd32f425::{self as pac, Pmu, Rcu};
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

    let mut time: f32 = 0.0;
    let mut ctr: u32 = 0;
    let mut sin: f32 = 0.0;

    loop {
        // application logic
        //
        // Increment the counter
        ctr += 1;
        time = ctr as f32 * 0.01;
    }
}

fn system_init() {
    // Create a handle to the RCU peripheral
    let rcu = unsafe { Rcu::steal() };
    // Create a handle to the PMU peripheral
    let pmu = unsafe { Pmu::steal() };

    // Set the IRC16MEN bit to enable the internal 16 MHz RC oscillator
    rcu.ctl0().modify(|_, w| w.irc16men().set_bit());

    rcu.cfg0()
        .modify(|_, w| unsafe { w.ahbpsc().bits(8 as u8) });

    // Spin for 50000 cycles
    for _ in 0..50000 {
        unsafe {
            asm!("nop");
        }
    }

    rcu.cfg0()
        .modify(|_, w| unsafe { w.ahbpsc().bits(9 as u8) });

    // Spin for 50000 cycles
    for _ in 0..50000 {
        unsafe {
            asm!("nop");
        }
    }

    // Configure CFG0
    // rcu.cfg0()
    //     .modify(|_, w| unsafe{ w.scs(0b00 as u8) });
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
    rcu.cfg0().reset();

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
    while rcu.ctl0().read().hxtalstb().bit_is_clear() {
        timeout += 1;
        if timeout > 100000 {
            break;
        }
    }

    // If that failed
    if timeout > 100000 {
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

    // Wait until PLL is selected as the system clock
    while rcu.cfg0().read().scss().bits() == 0b00 {}
}
