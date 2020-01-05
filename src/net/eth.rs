use crate::time::Instant;
use smoltcp::iface::EthernetInterface;
use smoltcp::socket::SocketSet;

pub const NEIGHBOR_CACHE_SIZE: usize = 32;
pub const SOCKET_BUFFER_SIZE: usize = 2048;

// TODO - socket management

pub struct Eth<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> {
    iface: EthernetInterface<'a, 'b, 'c, &'r mut stm32_eth::Eth<'rx, 'tx>>,
    sockets: SocketSet<'d, 'e, 'f>,
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> Eth<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> {
    pub fn new(
        iface: EthernetInterface<'a, 'b, 'c, &'r mut stm32_eth::Eth<'rx, 'tx>>,
        sockets: SocketSet<'d, 'e, 'f>,
    ) -> Self {
        Eth { iface, sockets }
    }

    pub fn poll(&mut self, time: Instant) {
        let t = smoltcp::time::Instant::from_millis(time.as_millis() as i64);
        match self.iface.poll(&mut self.sockets, t) {
            Ok(true) => (),
            _ => (),
        }
    }
}
