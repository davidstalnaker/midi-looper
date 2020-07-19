#![no_main]
#![no_std]

use cortex_m;
use panic_halt as _;
use stm32f4xx_hal as hal;

use crate::hal::{delay::*, gpio::*, prelude::*, stm32};

#[rtic::app(device = stm32)]
const APP: () = {
    struct Resources {
        delay: Delay,
        led: gpiod::PD14<Output<PushPull>>,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        let dp = stm32::Peripherals::take().unwrap();
        let cp = cortex_m::peripheral::Peripherals::take().unwrap();
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();
        let delay = Delay::new(cp.SYST, clocks);

        let gpiod = dp.GPIOD.split();

        let led = gpiod.pd14.into_push_pull_output();

        init::LateResources { delay, led }
    }

    #[idle(resources = [delay, led])]
    fn idle(c: idle::Context) -> ! {
        loop {
            c.resources.led.set_high().unwrap();
            c.resources.delay.delay_ms(1000_u32);
            c.resources.led.set_low().unwrap();
            c.resources.delay.delay_ms(1000_u32);
        }
    }
};
