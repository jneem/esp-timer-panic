#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use embassy_executor::_export::StaticCell;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use hal::prelude::*;
use hal::{clock::ClockControl, peripherals::Peripherals, timer::TimerGroup, Rtc};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::task]
async fn task() {
    loop {
        println!("ping");
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    hal::embassy::init(&clocks, timer_group0.timer0);

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| spawner.must_spawn(task()));
}
