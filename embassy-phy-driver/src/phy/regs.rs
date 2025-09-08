// Copied from: https://rust.docs.kernel.org/src/kernel/net/phy/reg.rs.html
// SPDX-License-Identifier: GPL-2.0

// Copyright (C) 2024 FUJITA Tomonori <fujita.tomonori@gmail.com>
//! PHY register interfaces.
//!
//! This module provides support for accessing PHY registers in the
//! Ethernet management interface clauses 22 and 45 register namespaces, as
//! defined in IEEE 802.3.

/// A single MDIO clause 22 register address (5 bits).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct C22(pub u8);

impl C22 {
    /// Basic mode control.
    pub const BMCR: Self = C22(0x00);
    /// Basic mode status.
    pub const BMSR: Self = C22(0x01);
    /// PHY identifier 1.
    pub const PHYSID1: Self = C22(0x02);
    /// PHY identifier 2.
    pub const PHYSID2: Self = C22(0x03);
    /// Auto-negotiation advertisement.
    pub const ADVERTISE: Self = C22(0x04);
    /// Auto-negotiation link partner base page ability.
    pub const LPA: Self = C22(0x05);
    /// Auto-negotiation expansion.
    pub const EXPANSION: Self = C22(0x06);
    /// Auto-negotiation next page transmit.
    pub const NEXT_PAGE_TRANSMIT: Self = C22(0x07);
    /// Auto-negotiation link partner received next page.
    pub const LP_RECEIVED_NEXT_PAGE: Self = C22(0x08);
    /// Master-slave control.
    pub const MASTER_SLAVE_CONTROL: Self = C22(0x09);
    /// Master-slave status.
    pub const MASTER_SLAVE_STATUS: Self = C22(0x0a);
    /// PSE Control.
    pub const PSE_CONTROL: Self = C22(0x0b);
    /// PSE Status.
    pub const PSE_STATUS: Self = C22(0x0c);
    /// MMD Register control.
    pub const MMD_CONTROL: Self = C22(0x0d);
    /// MMD Register address data.
    pub const MMD_DATA: Self = C22(0x0e);
    /// Extended status.
    pub const EXTENDED_STATUS: Self = C22(0x0f);
    /// Creates a new instance of `C22` with a vendor specific register.
    pub const fn vendor_specific<const N: u8>() -> Self {
        assert!(
            N > 0x0f && N < 0x20,
            "Vendor-specific register address must be between 16 and 31"
        );

        C22(N)
    }
}

/// A single MDIO clause 45 register device and address.
#[derive(Copy, Clone, Debug)]
pub struct Mmd(pub u8);

impl Mmd {
    /// Physical Medium Attachment/Dependent.
    pub const PMAPMD: Self = Mmd(1);
    /// WAN interface sublayer.
    pub const WIS: Self = Mmd(2);
    /// Physical coding sublayer.
    pub const PCS: Self = Mmd(3);
    /// PHY Extender sublayer.
    pub const PHYXS: Self = Mmd(4);
    /// DTE Extender sublayer.
    pub const DTEXS: Self = Mmd(5);
    /// Transmission convergence.
    pub const TC: Self = Mmd(6);
    /// Auto negotiation.
    pub const AN: Self = Mmd(7);
    /// Separated PMA (1).
    pub const SEPARATED_PMA1: Self = Mmd(8);
    /// Separated PMA (2).
    pub const SEPARATED_PMA2: Self = Mmd(9);
    /// Separated PMA (3).
    pub const SEPARATED_PMA3: Self = Mmd(10);
    /// Separated PMA (4).
    pub const SEPARATED_PMA4: Self = Mmd(11);
    /// OFDM PMA/PMD.
    pub const OFDM_PMAPMD: Self = Mmd(12);
    /// Power unit.
    pub const POWER_UNIT: Self = Mmd(13);
    /// Clause 22 extension.
    pub const C22_EXT: Self = Mmd(29);
    /// Vendor specific 1.
    pub const VEND1: Self = Mmd(30);
    /// Vendor specific 2.
    pub const VEND2: Self = Mmd(31);
}

/// A single MDIO clause 45 register device and address.
///
/// Clause 45 uses a 5-bit device address to access a specific MMD within
/// a port, then a 16-bit register address to access a location within
/// that device. `C45` represents this by storing a [`Mmd`] and
/// a register number.
#[derive(Debug, Clone, Copy)]
pub struct C45 {
    pub(crate) devad: Mmd,
    pub(crate) regnum: u16,
}

impl C45 {
    /// Creates a new instance of `C45`.
    pub const fn new(devad: Mmd, regnum: u16) -> Self {
        Self { devad, regnum }
    }
}
