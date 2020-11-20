#![no_main]
#![no_std]

mod clock;
mod loop_buffer;

use crate::hal::{gpio::*, prelude::*, serial::*, stm32, stm32::USART1, time::Bps};

use clock::Clock;
use cortex_m::{iprintln, peripheral::ITM};
use embedded_midi::{MidiIn, MidiMessage, MidiOut};
use heapless::{
    consts::*,
    i,
    spsc::{Consumer, Producer, Queue},
};
use loop_buffer::LoopBuffer;
use panic_itm as _;
use rtic::cyccnt::U32Ext;
use stm32f4xx_hal as hal;

const PERIOD: u32 = 168_000;

fn copy_midi_message(message: &MidiMessage) -> MidiMessage {
    return match message {
        &MidiMessage::NoteOff(channel, note, value) => MidiMessage::NoteOff(channel, note, value),
        &MidiMessage::NoteOn(channel, note, value) => MidiMessage::NoteOn(channel, note, value),
        &MidiMessage::KeyPressure(channel, note, value) => {
            MidiMessage::KeyPressure(channel, note, value)
        }
        &MidiMessage::ControlChange(channel, note, value) => {
            MidiMessage::ControlChange(channel, note, value)
        }
        &MidiMessage::ProgramChange(channel, program) => {
            MidiMessage::ProgramChange(channel, program)
        }
        &MidiMessage::ChannelPressure(channel, value) => {
            MidiMessage::ChannelPressure(channel, value)
        }
        &MidiMessage::PitchBendChange(channel, value) => {
            MidiMessage::PitchBendChange(channel, value)
        }
        &MidiMessage::QuarterFrame(quarter_frame) => MidiMessage::QuarterFrame(quarter_frame),
        &MidiMessage::SongPositionPointer(value) => MidiMessage::SongPositionPointer(value),
        &MidiMessage::SongSelect(value) => MidiMessage::SongSelect(value),
        &MidiMessage::TuneRequest => MidiMessage::TuneRequest,
        &MidiMessage::TimingClock => MidiMessage::TimingClock,
        &MidiMessage::Start => MidiMessage::Start,
        &MidiMessage::Continue => MidiMessage::Continue,
        &MidiMessage::Stop => MidiMessage::Stop,
        &MidiMessage::ActiveSensing => MidiMessage::ActiveSensing,
        &MidiMessage::Reset => MidiMessage::Reset,
    };
}

#[rtic::app(device = stm32, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        producer: Producer<'static, MidiMessage, U32>,
        consumer: Consumer<'static, MidiMessage, U32>,
        midiIn: MidiIn<Rx<USART1>>,
        midiOut: MidiOut<Tx<USART1>>,
        clock: Clock,
        loop_buffer: LoopBuffer,
        itm: ITM,
    }

    #[init(schedule = [tick])]
    fn init(c: init::Context) -> init::LateResources {
        static mut QUEUE: Queue<MidiMessage, U32> = Queue(i::Queue::new());
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

        let midiIn = MidiIn::new(rx);
        let midiOut = MidiOut::new(tx);

        let clock = Clock::new(1500);
        let loop_buffer = LoopBuffer::new();

        let mut core = c.core;
        core.DWT.enable_cycle_counter();
        c.schedule.tick(c.start + PERIOD.cycles()).unwrap();
        let mut itm = core.ITM;
        iprintln!(&mut itm.stim[0], "Start");

        init::LateResources {
            producer,
            consumer,
            midiIn,
            midiOut,
            clock,
            loop_buffer,
            itm,
        }
    }

    #[idle(resources = [consumer, midiOut, itm])]
    fn idle(mut c: idle::Context) -> ! {
        loop {
            if let Some(message) = c.resources.consumer.dequeue() {
                c.resources.midiOut.write(message).unwrap();
                c.resources.itm.lock(|itm| {
                    iprintln!(&mut itm.stim[0], "note");
                })
            }
        }
    }

    #[task(schedule = [tick], resources = [clock, producer, loop_buffer, itm])]
    fn tick(c: tick::Context) {
        c.resources.clock.increment();
        if c.resources.clock.get_current_count_ms() == 0 {
            iprintln!(&mut c.resources.itm.stim[0], "tick");
        }
        if let Some(message) = c
            .resources
            .loop_buffer
            .get_message(c.resources.clock.get_current_count_ms())
        {
            c.resources
                .producer
                .enqueue(copy_midi_message(message))
                .unwrap();
        }
        c.schedule.tick(c.scheduled + PERIOD.cycles()).unwrap();
    }

    #[task(binds = USART1, resources = [midiIn, producer, loop_buffer, clock, itm])]
    fn usart2(c: usart2::Context) {
        if let Ok(message) = c.resources.midiIn.read() {
            iprintln!(&mut c.resources.itm.stim[0], "note");
            c.resources
                .producer
                .enqueue(copy_midi_message(&message))
                .unwrap();
            c.resources
                .loop_buffer
                .insert_message(c.resources.clock.get_current_count_ms(), message);
        }
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SDIO();
    }
};
