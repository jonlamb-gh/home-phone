use crate::time::Instant;
use cortex_m::peripheral::syst::SystClkSource;
use hal::rcc::Clocks;
use hal::stm32::SYST;
use hal::time::Hertz;
use log::debug;

pub struct SysClock {
    ms: u64,
    frequency: Hertz,
}

impl SysClock {
    pub fn new(mut syst: SYST, clocks: Clocks) -> Self {
        debug!("Enable SystemClock freq {} Hz", clocks.hclk().0);

        // Generate an interrupt once a millisecond
        syst.set_clock_source(SystClkSource::External);
        syst.set_reload(clocks.hclk().0 / 8_000);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        // So the SYST can't be stopped or reset
        drop(syst);

        SysClock {
            ms: 0,
            frequency: clocks.sysclk(),
        }
    }

    pub fn frequency(&self) -> Hertz {
        self.frequency
    }

    pub fn set_time(&mut self, ms: u64) {
        self.ms = ms;
    }

    /// Time elapsed since `SystemClock` was created
    pub fn now(&self) -> Instant {
        Instant::from_millis(self.ms)
    }
}
