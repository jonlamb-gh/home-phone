#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_abort; // requires nightly
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr;
                         // requires a debugger

mod board_info;

use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::{entry, exception};
use log::{Level, LevelFilter, Metadata, Record};
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
use stm32_eth::{Eth, RingEntry};

// Pull in build information (from `built` crate)
mod build_info {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// TODO
const SRC_MAC: [u8; 6] = [0x00, 0x12, 0xDE, 0xAD, 0xBE, 0xEF];
const LOCAL_IP: Ipv4Address = Ipv4Address([192, 168, 1, 53]);
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

    stm32_eth::setup(&peripherals.RCC, &peripherals.SYSCFG);

    let mut flash = peripherals.FLASH.constrain();
    let mut rcc = peripherals.RCC.constrain();

    let mut gpioa = peripherals.GPIOA.split(&mut rcc.ahb1);
    let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
    let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb1);
    let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);
    let mut gpiog = peripherals.GPIOG.split(&mut rcc.ahb1);

    // TODO - fix the setup code
    //    stm32_eth::setup_pins(
    //        &mut gpioa, &mut gpiob, &mut gpioc, &mut gpiog, gpioa.pa1, gpioa.pa2,
    // gpioa.pa7,        gpiob.pb13, gpioc.pc1, gpioc.pc4, gpioc.pc5,
    // gpiog.pg11, gpiog.pg13,    );
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
    // PC1 RMII MDC - SB164 ON
    gpioc
        .pc1
        .into_af11(&mut gpioc.moder, &mut gpioc.afrl)
        .set_speed(VeryHigh);
    // PA7 RMII RX Data Valid D11 JP6 ON
    gpioa
        .pa7
        .into_af11(&mut gpioa.moder, &mut gpioa.afrl)
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
    // PB13 RMII TXD1 I2S_A_CK JP7 ON
    gpiob
        .pb13
        .into_af11(&mut gpiob.moder, &mut gpiob.afrh)
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

    // TODO - revist my RCC bits
    // Default clock configuration runs at 16 MHz
    //let _clocks = rcc.cfgr.freeze(&mut flash.acr);
    let clocks = rcc.cfgr.freeze_max(&mut flash.acr);

    let mut leds = Leds::new(led_r, led_g, led_b);
    for led in leds.iter_mut() {
        //led.on();
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

    setup_systick(&mut core_peripherals.SYST);

    let mut rx_ring: [RingEntry<_>; 8] = Default::default();
    let mut tx_ring: [RingEntry<_>; 2] = Default::default();
    let mut eth = Eth::new(
        peripherals.ETHERNET_MAC,
        peripherals.ETHERNET_DMA,
        &mut rx_ring[..],
        &mut tx_ring[..],
    );
    eth.enable_interrupt(&mut core_peripherals.NVIC);

    let ip_addr = IpCidr::new(IpAddress::from(LOCAL_IP), 24);
    let mut ip_addrs = [ip_addr];
    let mut neighbor_storage = [None; 16];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let ethernet_addr = EthernetAddress(SRC_MAC);
    let mut iface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .neighbor_cache(neighbor_cache)
        .finalize();

    let mut server_rx_buffer = [0; 2048];
    let mut server_tx_buffer = [0; 2048];
    let server_socket = TcpSocket::new(
        TcpSocketBuffer::new(&mut server_rx_buffer[..]),
        TcpSocketBuffer::new(&mut server_tx_buffer[..]),
    );
    let mut sockets_storage = [None, None];
    let mut sockets = SocketSet::new(&mut sockets_storage[..]);
    let server_handle = sockets.add(server_socket);

    // TODO
    let mut _timer = Timer::tim2(peripherals.TIM2, 1.hz(), clocks, &mut rcc.apb1);

    loop {
        let time: u64 = cortex_m::interrupt::free(|cs| *TIME.borrow(cs).borrow());
        cortex_m::interrupt::free(|cs| {
            let mut eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
            *eth_pending = false;
        });
        match iface.poll(&mut sockets, Instant::from_millis(time as i64)) {
            Ok(true) => {
                leds[Color::Green].toggle();
                let mut socket = sockets.get::<TcpSocket>(server_handle);
                if !socket.is_open() {
                    socket
                        .listen(TCP_PORT)
                        .or_else(|e| writeln!(debug_console, "TCP listen error: {:?}", e))
                        .unwrap();
                }

                if socket.can_send() {
                    write!(socket, "hello\n")
                        .map(|_| {
                            socket.close();
                        }).or_else(|e| writeln!(debug_console, "TCP send error: {:?}", e))
                        .unwrap();
                }
            }
            Ok(false) => {
                // Sleep if no ethernet work is pending
                cortex_m::interrupt::free(|cs| {
                    let eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
                    if !*eth_pending {
                        asm::wfi();
                        // Awaken by interrupt
                    }
                });
            }
            Err(e) =>
            // Ignore malformed packets
            {
                writeln!(debug_console, "Error: {:?}", e).unwrap()
            }
        }
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

//#[interrupt]
interrupt!(ETH, eth_isr);

//fn ETH() {
fn eth_isr() {
    cortex_m::interrupt::free(|cs| {
        let mut eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
        *eth_pending = true;
    });

    // Clear interrupt flags
    let p = unsafe { stm32f7x7::Peripherals::steal() };
    stm32_eth::eth_interrupt_handler(&p.ETHERNET_DMA);
}
