#![no_main]
#![no_std]

mod clock;

use crate::hal::{gpio::*, prelude::*, serial::*, stm32, stm32::USART1, time::Bps};

use clock::Clock;
use heapless::{
    consts::*,
    i,
    spsc::{Consumer, Producer, Queue},
};
use midi_port::{MidiInPort, MidiMessage};
use panic_halt as _;
use rtic::cyccnt::U32Ext;
use stm32f4xx_hal as hal;

const PERIOD: u32 = 168_000;

#[rtic::app(device = stm32, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        producer: Producer<'static, u8, U32>,
        consumer: Consumer<'static, u8, U32>,
        midiIn: MidiInPort<Rx<USART1>>,
        tx: Tx<USART1>,
        clock: Clock,
    }

    #[init(schedule = [tick])]
    fn init(c: init::Context) -> init::LateResources {
        static mut QUEUE: Queue<u8, U32> = Queue(i::Queue::new());
        let (producer, consumer) = QUEUE.split();

        let dp = stm32::Peripherals::take().unwrap();
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

        let _gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();

        let txPin = gpiob.pb6.into_alternate_af7();
        let rxPin = gpiob.pb7.into_alternate_af7();
        let mut serial = Serial::usart1(
            dp.USART1,
            (txPin, rxPin),
            config::Config::default().baudrate(Bps(31250)),
            clocks,
        )
        .unwrap();
        serial.listen(Event::Rxne);
        let (tx, rx) = serial.split();

        let midiIn = MidiInPort::new(rx);

        let clock = Clock::new();

        let mut core = c.core;
        core.DWT.enable_cycle_counter();
        c.schedule.tick(c.start + PERIOD.cycles()).unwrap();

        init::LateResources {
            producer,
            consumer,
            midiIn,
            tx,
            clock,
        }
    }

    #[idle(resources = [consumer, tx])]
    fn idle(c: idle::Context) -> ! {
        loop {
            if let Some(byte) = c.resources.consumer.peek() {
                if c.resources.tx.write(*byte).is_ok() {
                    c.resources.consumer.dequeue().unwrap();
                }
            }
        }
    }

    #[task(schedule = [tick], resources = [clock, producer])]
    fn tick(c: tick::Context) {
        c.resources.clock.increment();
        if c.resources.clock.count % 1000 == 0 {
            let bytes = b"tick\r\n";
            for b in bytes {
                c.resources.producer.enqueue(*b).unwrap();
            }
        }
        c.schedule.tick(c.scheduled + PERIOD.cycles()).unwrap();
    }

    #[task(binds = USART1, resources = [producer, midiIn])]
    fn usart2(c: usart2::Context) {
        c.resources.midiIn.poll_uart();
        if let Some(message) = c.resources.midiIn.get_message() {
            match message {
                MidiMessage::NoteOn { .. } => {
                    let bytes = b"note on\r\n";
                    for b in bytes {
                        c.resources.producer.enqueue(*b).unwrap();
                    }
                }
                _ => {}
            }
        }
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SDIO();
    }
};
