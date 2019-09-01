#![no_std]
#![no_main]

// pick a panicking behavior
//extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
extern crate panic_semihosting; // logs messages to the host stderr;
                                // requires a debugger

mod board_info;
mod eth;

use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::{entry, exception};
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::gpio::Speed::VeryHigh;
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::serial::Serial;
use nucleo_f767zi::hal::stm32f7x7::{self, interrupt, SYST};
use nucleo_f767zi::hal::timer::Timer;
use nucleo_f767zi::led::{Color, Leds};
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};

// Pull in build information (from `built` crate)
mod build_info {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// TODO
const SRC_MAC: [u8; 6] = [0x02, 0x00, 0x05, 0x06, 0x07, 0x08];
const LOCAL_IP: Ipv4Address = Ipv4Address([192, 168, 1, 39]);
const TCP_PORT: u16 = 1234;

static TIME: Mutex<RefCell<u64>> = Mutex::new(RefCell::new(0));
static ETH_PENDING: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

#[entry]
fn main() -> ! {
    let mut core_peripherals =
        cortex_m::Peripherals::take().expect("Failed to take cortex_m::Peripherals");
    let peripherals =
        stm32f7x7::Peripherals::take().expect("Failed to take stm32f7x7::Peripherals");

    core_peripherals.SCB.enable_icache();
    core_peripherals
        .SCB
        .enable_dcache(&mut core_peripherals.CPUID);

    let mut flash = peripherals.FLASH.constrain();
    let mut rcc = peripherals.RCC.constrain();

    // TODO - revist my RCC bits
    // set up PLL to 168MHz from 16MHz HSI
    // Default clock configuration runs at 16 MHz
    //let clocks = rcc.cfgr.freeze(&mut flash.acr);
    //
    //let clocks = rcc.cfgr.freeze_max(&mut flash.acr);
    //
    let clocks = rcc.cfgr.sysclk(168.mhz()).freeze(&mut flash.acr);

    //let clocks = rcc.cfgr.sysclk(32.mhz()).freeze(&mut flash.acr);

    let mut gpioa = peripherals.GPIOA.split(&mut rcc.ahb1);
    let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
    let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb1);
    let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);
    let mut gpiog = peripherals.GPIOG.split(&mut rcc.ahb1);

    // PA1 RMII Reference Clock - SB13 ON
    gpioa
        .pa1
        .into_af11(&mut gpioa.moder, &mut gpioa.afrl)
        .set_speed(VeryHigh);
    // PA2 RMII MDIO - SB160 ON
    gpioa
        .pa2
        .into_af11(&mut gpioa.moder, &mut gpioa.afrl)
        .set_speed(VeryHigh);
    // PA7 RMII RX Data Valid D11 JP6 ON
    gpioa
        .pa7
        .into_af11(&mut gpioa.moder, &mut gpioa.afrl)
        .set_speed(VeryHigh);

    // PB13 RMII TXD1 I2S_A_CK JP7 ON
    gpiob
        .pb13
        .into_af11(&mut gpiob.moder, &mut gpiob.afrh)
        .set_speed(VeryHigh);

    // PC1 RMII MDC - SB164 ON
    gpioc
        .pc1
        .into_af11(&mut gpioc.moder, &mut gpioc.afrl)
        .set_speed(VeryHigh);
    // PC4 RMII RXD0 - SB178 ON
    gpioc
        .pc4
        .into_af11(&mut gpioc.moder, &mut gpioc.afrl)
        .set_speed(VeryHigh);
    // PC5 RMII RXD1 - SB181 ON
    gpioc
        .pc5
        .into_af11(&mut gpioc.moder, &mut gpioc.afrl)
        .set_speed(VeryHigh);

    // PG2
    gpiog
        .pg2
        .into_af11(&mut gpiog.moder, &mut gpiog.afrl)
        .set_speed(VeryHigh);
    // PG11 RMII TX Enable - SB183 ON
    gpiog
        .pg11
        .into_af11(&mut gpiog.moder, &mut gpiog.afrh)
        .set_speed(VeryHigh);
    // PG13 RXII TXD0 - SB182 ON
    gpiog
        .pg13
        .into_af11(&mut gpiog.moder, &mut gpiog.afrh)
        .set_speed(VeryHigh);

    let led_r = gpiob
        .pb14
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let led_g = gpiob
        .pb0
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let led_b = gpiob
        .pb7
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let usart3_tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
    let usart3_rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);

    let mut leds = Leds::new(led_r, led_g, led_b);
    for led in leds.iter_mut() {
        led.off();
    }

    // USART3 is routed up to the same USB port as the stlink
    let serial = Serial::usart3(
        peripherals.USART3,
        (usart3_tx, usart3_rx),
        115_200.bps(),
        clocks,
        &mut rcc.apb1,
    );

    let mut debug_console = DebugConsole::new(serial);

    writeln!(debug_console, "Board Initialized").ok();
    board_info::build_info(&mut debug_console);

    writeln!(
        debug_console,
        "MAC Address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        SRC_MAC[0], SRC_MAC[1], SRC_MAC[2], SRC_MAC[3], SRC_MAC[4], SRC_MAC[5]
    )
    .ok();

    let mac_addr = smoltcp::wire::EthernetAddress::from_bytes(&SRC_MAC);
    let mut ethdev = eth::EthernetDevice::new(peripherals.ETHERNET_MAC, peripherals.ETHERNET_DMA);
    // TODO
    //ethdev.init(&mut peripherals.RCC, mac_addr);
    let mut stolen_p = unsafe { stm32f7x7::Peripherals::steal() };
    ethdev.init(&mut stolen_p.RCC, &mut stolen_p.SYSCFG, mac_addr);

    writeln!(debug_console, "Ethernet Initialized").ok();

    ethdev.enable_interrupt();

    writeln!(debug_console, "Waiting for link").ok();

    ethdev.block_until_link();

    writeln!(debug_console, "Link up").ok();

    setup_systick(&mut core_peripherals.SYST);

    loop {
        let time: u64 = cortex_m::interrupt::free(|cs| *TIME.borrow(cs).borrow());
        cortex_m::interrupt::free(|cs| {
            let mut eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
            *eth_pending = false;
        });

        //writeln!(debug_console, "Time {}", time).ok();
        leds[Color::Blue].toggle();
    }
}

fn setup_systick(syst: &mut SYST) {
    syst.set_reload(SYST::get_ticks_per_10ms() / 10);
    syst.enable_counter();
    syst.enable_interrupt();
}

#[exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        let mut time = TIME.borrow(cs).borrow_mut();
        *time += 1;
    })
}

#[interrupt]
fn ETH() {
    cortex_m::interrupt::free(|cs| {
        let mut eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
        *eth_pending = true;
    });

    // Clear interrupt flags
    let mut p = unsafe { stm32f7x7::Peripherals::steal() };
    eth::EthernetDevice::eth_interrupt_handler(&mut p.ETHERNET_DMA);
}
