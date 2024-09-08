#![no_std]
#![no_main]

use cortex_m_rt::entry;
use gd32f4::gd32f425::{self as pac, Rcu};
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
        sin = sinf(time);
    }
}

fn system_init() {
    use pac::rcu::*;
    // Create a handle to the RCU peripheral
    let rcu = unsafe { Rcu::steal() };

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

    // Set the RCU_CFG0 register to select the internal 16 MHz RC oscillator as the system clock
}
