#![no_std]
#![no_main]
#![cfg_attr(not(test), feature(core_intrinsics))]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(test_runner::runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

use core::str;
use lib::hal::bcm2711::gpio::GPIO;
use lib::hal::bcm2711::mbox::MBOX;
use lib::hal::bcm2711::uart1::UART1;
use lib::hal::clocks::Clocks;
use lib::hal::mailbox::*;
use lib::hal::prelude::*;
use lib::hal::serial::Serial;
use lib::hal::time::Bps;
use lib::logger::Logger;
use lib::net::loopback::Loopback;
use log::{debug, error, trace, LevelFilter};
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};

static GLOBAL_LOGGER: Logger = Logger::new();

raspi3_boot::entry!(test_entry);
fn test_entry() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();
    let gp = gpio.split();

    let tx = gp.p14.into_alternate_af5();
    let rx = gp.p15.into_alternate_af5();

    let serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

    GLOBAL_LOGGER.set_inner(serial);
    unsafe {
        log::set_logger_racy(&GLOBAL_LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .unwrap();
    }

    crate::test_main();

    qemu::exit(qemu::ExitCode::Success)
}

mod mock {
    use core::cell::Cell;
    use smoltcp::time::{Duration, Instant};

    #[derive(Debug)]
    pub struct Clock(Cell<Instant>);

    impl Clock {
        pub fn new() -> Clock {
            Clock(Cell::new(Instant::from_millis(0)))
        }

        pub fn advance(&self, duration: Duration) {
            self.0.set(self.0.get() + duration);
        }

        pub fn elapsed(&self) -> Instant {
            self.0.get()
        }
    }
}

#[test_case]
fn tcp_send_recv() {
    trace!("tcp_send_recv");
    let clock = mock::Clock::new();
    let device = Loopback::new();

    // Build the IP stack
    let ip_addr = IpCidr::new(IpAddress::v4(127, 0, 0, 1), 24);
    let mut ip_addrs = [ip_addr];

    // Up to 1024 ARP (neighbor) cache entries
    let mut neighbor_storage = [None; 1024];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);

    let ethernet_addr = EthernetAddress::default();
    let mut iface = EthernetInterfaceBuilder::new(device)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .neighbor_cache(neighbor_cache)
        .finalize();

    // Buffers statically allocated
    let server_socket = {
        static mut TCP_SERVER_RX_DATA: [u8; 1024] = [0; 1024];
        static mut TCP_SERVER_TX_DATA: [u8; 1024] = [0; 1024];
        let tcp_rx_buffer = TcpSocketBuffer::new(unsafe { &mut TCP_SERVER_RX_DATA[..] });
        let tcp_tx_buffer = TcpSocketBuffer::new(unsafe { &mut TCP_SERVER_TX_DATA[..] });
        TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer)
    };

    let client_socket = {
        static mut TCP_CLIENT_RX_DATA: [u8; 1024] = [0; 1024];
        static mut TCP_CLIENT_TX_DATA: [u8; 1024] = [0; 1024];
        let tcp_rx_buffer = TcpSocketBuffer::new(unsafe { &mut TCP_CLIENT_RX_DATA[..] });
        let tcp_tx_buffer = TcpSocketBuffer::new(unsafe { &mut TCP_CLIENT_TX_DATA[..] });
        TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer)
    };

    let mut sockets_storage = [None, None];
    let mut sockets = SocketSet::new(&mut sockets_storage[..]);

    let server_handle = sockets.add(server_socket);
    let client_handle = sockets.add(client_socket);

    let mut did_listen = false;
    let mut did_connect = false;
    let mut done = false;

    let payload = b"0123456789abcdef";

    while !done && clock.elapsed() < Instant::from_secs(10) {
        match iface.poll(&mut sockets, clock.elapsed()) {
            Ok(_) => {}
            Err(e) => {
                debug!("Poll error: {}", e);
            }
        }

        {
            let mut socket = sockets.get::<TcpSocket>(server_handle);
            if !socket.is_active() && !socket.is_listening() {
                if !did_listen {
                    debug!("Listening");
                    socket.listen(1234).unwrap();
                    did_listen = true;
                }
            }

            if socket.can_recv() {
                let recvd_payload =
                    socket.recv(|buffer| (buffer.len(), str::from_utf8(buffer).unwrap()));
                debug!("Got {:?}", recvd_payload);
                let recvd_payload = recvd_payload.unwrap();
                assert_eq!(recvd_payload, str::from_utf8(payload).unwrap());
                socket.close();
                done = true;
            }
        }
        {
            let mut socket = sockets.get::<TcpSocket>(client_handle);
            if !socket.is_open() {
                if !did_connect {
                    debug!("Connecting");
                    socket
                        .connect(
                            (IpAddress::v4(127, 0, 0, 1), 1234),
                            (IpAddress::Unspecified, 65000),
                        )
                        .unwrap();
                    did_connect = true;
                }
            }

            if socket.can_send() {
                debug!("Sending");
                socket.send_slice(payload).unwrap();
                socket.close();
            }
        }

        match iface.poll_delay(&sockets, clock.elapsed()) {
            Some(Duration { millis: 0 }) => {}
            Some(delay) => {
                debug!("Sleeping for {} ms", delay);
                clock.advance(delay)
            }
            None => clock.advance(Duration::from_millis(1)),
        }
    }

    if done {
        debug!("Done")
    } else {
        error!("Took too long to complete")
    }

    assert_eq!(done, true);
}
