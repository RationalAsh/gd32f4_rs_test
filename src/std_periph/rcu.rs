use gd32f4::gd32f425::Rcu;

pub fn rcu_periph_clock_enable() {
    let rcu = unsafe { Rcu::steal() };

    // Enable the GPIOA peripheral
    rcu.ahb1en().modify(|_, w| w.paen().set_bit());
    // Enable the GPIOB peripheral
    rcu.ahb1en().modify(|_, w| w.pben().set_bit());
    // Enable the GPIOC peripheral
    rcu.ahb1en().modify(|_, w| w.pcen().set_bit());
}
