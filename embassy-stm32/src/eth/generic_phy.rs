//! Generic SMI Ethernet PHY

use core::task::Context;

use embassy_phy_driver::{
    phy::{
        regs::{Mmd, C22, C45},
        DuplexMode, Speed,
    },
    StationManagement,
};
#[cfg(feature = "time")]
use embassy_time::{Duration, Timer};
#[cfg(feature = "time")]
use futures_util::FutureExt;

use crate::eth::PhyLink;

use super::Phy;

#[allow(dead_code)]
mod phy_consts {
    pub const PHY_REG_WUCSR: u16 = 0x8010;

    pub const PHY_REG_BCR_COLTEST: u16 = 1 << 7;
    pub const PHY_REG_BCR_FD: u16 = 1 << 8;
    pub const PHY_REG_BCR_ANRST: u16 = 1 << 9;
    pub const PHY_REG_BCR_ISOLATE: u16 = 1 << 10;
    pub const PHY_REG_BCR_POWERDN: u16 = 1 << 11;
    pub const PHY_REG_BCR_AN: u16 = 1 << 12;
    pub const PHY_REG_BCR_100M: u16 = 1 << 13;
    pub const PHY_REG_BCR_LOOPBACK: u16 = 1 << 14;
    pub const PHY_REG_BCR_RESET: u16 = 1 << 15;

    pub const PHY_REG_BSR_JABBER: u16 = 1 << 1;
    pub const PHY_REG_BSR_UP: u16 = 1 << 2;
    pub const PHY_REG_BSR_FAULT: u16 = 1 << 4;
    pub const PHY_REG_BSR_ANDONE: u16 = 1 << 5;
}
use self::phy_consts::*;

/// Generic SMI Ethernet PHY implementation
pub struct GenericPhy {
    phy_addr: u8,
    #[cfg(feature = "time")]
    poll_interval: Duration,
}

impl GenericPhy {
    /// Construct the PHY. It assumes the address `phy_addr` in the SMI communication
    ///
    /// # Panics
    /// `phy_addr` must be in range `0..32`
    pub fn new(phy_addr: u8) -> Self {
        assert!(phy_addr < 32);
        Self {
            phy_addr,
            #[cfg(feature = "time")]
            poll_interval: Duration::from_millis(500),
        }
    }

    /// Construct the PHY. Try to probe all addresses from 0 to 31 during initialization
    ///
    /// # Panics
    /// Initialization panics if PHY didn't respond on any address
    pub fn new_auto() -> Self {
        Self {
            phy_addr: 0xFF,
            #[cfg(feature = "time")]
            poll_interval: Duration::from_millis(500),
        }
    }
}

// TODO: Factor out to shared functionality
fn blocking_delay_us(us: u32) {
    #[cfg(feature = "time")]
    embassy_time::block_for(Duration::from_micros(us as u64));
    #[cfg(not(feature = "time"))]
    {
        let freq = unsafe { crate::rcc::get_freqs() }.sys.to_hertz().unwrap().0 as u64;
        let us = us as u64;
        let cycles = freq * us / 1_000_000;
        cortex_m::asm::delay(cycles as u32);
    }
}

impl Phy for GenericPhy {
    fn phy_reset<S: StationManagement>(&mut self, sm: &mut S) -> Result<(), S::Error> {
        // Detect SMI address
        if self.phy_addr == 0xFF {
            for addr in 0..32 {
                sm.smi_write(addr, C22::BMCR, PHY_REG_BCR_RESET)?;
                for _ in 0..10 {
                    if sm.smi_read(addr, C22::BMCR)? & PHY_REG_BCR_RESET != PHY_REG_BCR_RESET {
                        trace!("Found ETH PHY on address {}", addr);
                        self.phy_addr = addr;
                        return Ok(());
                    }
                    // Give PHY a total of 100ms to respond
                    blocking_delay_us(10000);
                }
            }
            panic!("PHY did not respond");
        }

        sm.smi_write(self.phy_addr, C22::BMCR, PHY_REG_BCR_RESET)?;

        while sm.smi_read(self.phy_addr, C22::BMCR)? & PHY_REG_BCR_RESET == PHY_REG_BCR_RESET {}

        Ok(())
    }

    fn phy_init<S: StationManagement>(&mut self, sm: &mut S) -> Result<(), S::Error> {
        // Clear WU CSR
        sm.smi_write_mmd(self.phy_addr, C45::new(Mmd::PCS, PHY_REG_WUCSR), 0)?;

        // Enable auto-negotiation
        sm.smi_write(
            self.phy_addr,
            C22::BMCR,
            PHY_REG_BCR_AN | PHY_REG_BCR_ANRST | PHY_REG_BCR_100M,
        )
    }

    fn poll_link<S: StationManagement>(&mut self, sm: &mut S, cx: &mut Context) -> Result<PhyLink, S::Error> {
        #[cfg(not(feature = "time"))]
        cx.waker().wake_by_ref();

        #[cfg(feature = "time")]
        let _ = Timer::after(self.poll_interval).poll_unpin(cx);

        let bmsr = sm.smi_read(self.phy_addr, C22::BMSR)?;

        // No link without autonegotiate
        if bmsr & PHY_REG_BSR_ANDONE == 0 {
            return Ok(PhyLink::Down);
        }
        // No link if link is down
        if bmsr & PHY_REG_BSR_UP == 0 {
            return Ok(PhyLink::Down);
        }

        let advertising = sm.smi_read(self.phy_addr, C22::ADVERTISE)?;
        let lpa = sm.smi_read(self.phy_addr, C22::LPA)?;

        let nego = advertising & lpa;

        Ok(if nego & 0x0100 != 0 {
            PhyLink::Up {
                speed: Speed::_100,
                duplex: DuplexMode::Full,
            }
        } else if nego & 0x0080 != 0 {
            PhyLink::Up {
                speed: Speed::_100,
                duplex: DuplexMode::Half,
            }
        } else if nego & 0x0040 != 0 {
            PhyLink::Up {
                speed: Speed::_10,
                duplex: DuplexMode::Full,
            }
        } else {
            PhyLink::Up {
                speed: Speed::_10,
                duplex: DuplexMode::Half,
            }
        })
    }
}

/// Public functions for the PHY
impl GenericPhy {
    /// Set the SMI polling interval.
    #[cfg(feature = "time")]
    pub fn set_poll_interval(&mut self, poll_interval: Duration) {
        self.poll_interval = poll_interval
    }
}
