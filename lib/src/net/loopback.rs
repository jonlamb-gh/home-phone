//! Loopback PhyDevice

use arrayvec::ArrayVec;
use heapless::Vec;
use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;
use smoltcp::{Error, Result};
use typenum::{Sum, Unsigned, U1024, U476};

/// Default MTU size is 1,500 bytes
/// U1024 + U476 = U1500
pub type MtuSize = Sum<U1024, U476>;

const QLEN: usize = 32;

type QElement = Vec<u8, MtuSize>;

#[derive(Debug)]
pub struct Loopback {
    queue: ArrayVec<[QElement; QLEN]>,
}

impl Loopback {
    pub fn new() -> Loopback {
        Loopback {
            queue: ArrayVec::new(),
        }
    }
}

impl<'a> Device<'a> for Loopback {
    type RxToken = RxToken;
    type TxToken = TxToken<'a>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MtuSize::USIZE;
        caps
    }

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        self.queue.pop_at(0).map(move |buffer| {
            let rx = RxToken { buffer: buffer };
            let tx = TxToken {
                queue: &mut self.queue,
            };
            (rx, tx)
        })
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(TxToken {
            queue: &mut self.queue,
        })
    }
}

pub struct RxToken {
    buffer: QElement,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        f(&mut self.buffer)
    }
}

pub struct TxToken<'a> {
    queue: &'a mut ArrayVec<[QElement; QLEN]>,
}

impl<'a> phy::TxToken for TxToken<'a> {
    fn consume<R, F>(self, _timestamp: Instant, len: usize, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        let mut buffer = QElement::new();
        buffer.resize(len, 0).map_err(|_| Error::Exhausted)?;
        let result = f(&mut buffer);
        self.queue.try_push(buffer).map_err(|_| Error::Exhausted)?;
        result
    }
}
