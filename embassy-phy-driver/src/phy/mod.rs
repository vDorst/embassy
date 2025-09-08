use core::task::Context;

use crate::StationManagement;

pub mod regs;

/// Link Speed
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    /// 10 MBit
    _10,
    /// 100 MBit
    _100,
    /// 100 MBit
    _1000,
    /// 2500 MBit
    _2500,
    /// 5000 MBit
    _5000,
    /// 100000 MBit
    _10000,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Duplex
pub enum DuplexMode {
    /// Full
    Full,
    /// Half
    Half,
}

/// Link Status
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LinkStatus {
    /// Link Down
    Down,
    /// Link Up with `Speed` and `Duplex`
    Up {
        /// Link speed
        speed: Speed,
        /// Link Duplex
        duplex: DuplexMode,
    },
}

impl LinkStatus {
    /// Is link up
    pub fn is_up(&self) -> bool {
        matches!(self, Self::Up { speed: _, duplex: _ })
    }
    /// Is link down
    pub fn is_down(&self) -> bool {
        matches!(self, Self::Down)
    }
}

/// Trait for an Ethernet PHY
pub trait Phy {
    /// Reset PHY and wait for it to come out of reset.
    fn phy_reset<S: StationManagement>(&mut self, sm: &mut S) -> Result<(), S::Error>;
    /// PHY initialisation.
    fn phy_init<S: StationManagement>(&mut self, sm: &mut S) -> Result<(), S::Error>;
    /// Poll link to see if it is up and FD with 100Mbps
    fn poll_link<S: StationManagement>(&mut self, sm: &mut S, cx: &mut Context) -> Result<LinkStatus, S::Error>;
}
