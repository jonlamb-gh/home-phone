#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use core::cell::Cell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use lib::hal::prelude::*;
use lib::hal::serial::{config::Config, Serial};
use lib::hal::stm32::{self, interrupt};
use lib::logger::Logger;
use lib::net::eth::{Eth, NEIGHBOR_CACHE_SIZE, SOCKET_BUFFER_SIZE};
use lib::sys_clock::SysClock;
use log::{debug, info, LevelFilter};
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::wire::{EthernetAddress, IpCidr, Ipv4Address};

mod panic_handler;

const SRC_MAC: [u8; 6] = [0x02, 0x00, 0x05, 0x06, 0x07, 0x08];
const SRC_IP: [u8; 4] = [192, 168, 1, 39];

static GLOBAL_LOGGER: Logger = Logger::new();

static GLOBAL_SYST_MS: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

static GLOBAL_ETH_PENDING: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");
    let mut cp =
        cortex_m::peripheral::Peripherals::take().expect("Failed to take cortex_m::Peripherals");

    stm32_eth::setup(&dp.RCC, &dp.SYSCFG);

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(180.mhz()).freeze();

    let gpiod = dp.GPIOD.split();
    let pin_tx = gpiod.pd8.into_alternate_af7();
    let pin_rx = gpiod.pd9.into_alternate_af7();

    let serial = Serial::usart3(
        dp.USART3,
        (pin_tx, pin_rx),
        Config {
            baudrate: 115_200.bps(),
            ..Default::default()
        },
        clocks,
    )
    .unwrap();

    // Setup logger on USART3
    let (tx, _rx) = serial.split();
    GLOBAL_LOGGER.set_inner(tx);
    log::set_logger(&GLOBAL_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);

    debug!("Setup Ethernet");
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiog = dp.GPIOG.split();
    stm32_eth::setup_pins(
        gpioa.pa1, gpioa.pa2, gpioa.pa7, gpiob.pb13, gpioc.pc1, gpioc.pc4, gpioc.pc5, gpiog.pg11,
        gpiog.pg13,
    );

    let mut rx_ring: [stm32_eth::RingEntry<_>; 16] = Default::default();
    let mut tx_ring: [stm32_eth::RingEntry<_>; 8] = Default::default();
    let mut eth = stm32_eth::Eth::new(
        dp.ETHERNET_MAC,
        dp.ETHERNET_DMA,
        SRC_MAC,
        &mut rx_ring[..],
        &mut tx_ring[..],
    );
    eth.enable_interrupt(&mut cp.NVIC);

    debug!("Setup IP stack");
    let ip = Ipv4Address::from_bytes(&SRC_IP);
    let mac = EthernetAddress::from_bytes(&SRC_MAC);
    info!("{}, {}", ip, mac);
    let ip_addr = IpCidr::new(ip.into(), 24);
    let mut ip_addrs = [ip_addr];
    let mut neighbor_storage = [None; NEIGHBOR_CACHE_SIZE];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let mut routes_storage = [];
    let routes = Routes::new(&mut routes_storage[..]);
    let iface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(mac.into())
        .ip_addrs(&mut ip_addrs[..])
        .neighbor_cache(neighbor_cache)
        .routes(routes)
        .finalize();

    // TODO - only 1 socket for now, move this to the Eth area
    let mut sockets_storage = [None];
    let mut sockets = SocketSet::new(&mut sockets_storage[..]);

    let tcp_server_socket = {
        static mut RX_BUFFER: [u8; SOCKET_BUFFER_SIZE] = [0; SOCKET_BUFFER_SIZE];
        static mut TX_BUFFER: [u8; SOCKET_BUFFER_SIZE] = [0; SOCKET_BUFFER_SIZE];
        TcpSocket::new(
            TcpSocketBuffer::new(unsafe { &mut RX_BUFFER[..] }),
            TcpSocketBuffer::new(unsafe { &mut TX_BUFFER[..] }),
        )
    };

    let server_handle = sockets.add(tcp_server_socket);

    let mut eth = Eth::new(iface, sockets);

    let mut sys_clock = SysClock::new(cp.SYST, clocks);

    let mut last_sec = 0;
    loop {
        let ms: u64 = cortex_m::interrupt::free(|cs| GLOBAL_SYST_MS.borrow(cs).get());
        sys_clock.set_time(ms);
        let time = sys_clock.now();

        let eth_pending =
            cortex_m::interrupt::free(|cs| GLOBAL_ETH_PENDING.borrow(cs).replace(false));
        if eth_pending {
            eth.poll(time);
        }

        let sec = time.as_secs();
        if sec != last_sec {
            info!("{}", lib::time::DisplayableInstant::from(time));
            last_sec = sec;
        }
    }
}

#[exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        let cell = GLOBAL_SYST_MS.borrow(cs);
        let t = cell.get();
        cell.replace(t.wrapping_add(1));
    })
}

#[interrupt]
fn ETH() {
    cortex_m::interrupt::free(|cs| {
        GLOBAL_ETH_PENDING.borrow(cs).replace(true);
    });

    // Clear interrupt flags
    let p = unsafe { stm32::Peripherals::steal() };
    stm32_eth::eth_interrupt_handler(&p.ETHERNET_DMA);
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
