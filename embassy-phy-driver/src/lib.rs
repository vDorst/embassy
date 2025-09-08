#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use crate::phy::regs::{C22, C45};
use core::future::Future;

/// Phy
pub mod phy;

#[allow(dead_code)]
#[repr(u16)]
enum Reg13Op {
    Addr = 0b00 << 14,
    Write = 0b01 << 14,
    PostReadIncAddr = 0b10 << 14,
    Read = 0b11 << 14,
}
#[allow(dead_code)]
const DEV_MASK: u8 = 0x1f;

/// Station Management Interface (SMI) on an ethernet PHY
pub trait StationManagement {
    /// `StationManagement` error type
    type Error: core::fmt::Debug;

    /// Read a register over SMI.
    fn smi_read(&mut self, phy_addr: u8, reg: C22) -> Result<u16, Self::Error>;
    /// Write a register over SMI.
    fn smi_write(&mut self, phy_addr: u8, reg: C22, val: u16) -> Result<(), Self::Error>;

    /// Read, Clause 45
    /// This is the default implementation.
    /// Many hardware these days support direct Clause 45 operations.
    /// Implement this function when your hardware supports it.
    fn smi_read_mmd(&mut self, phy_addr: u8, reg: C45) -> Result<u16, Self::Error> {
        let devad = u16::from(reg.devad.0 & DEV_MASK);

        // Write FN
        let val = (Reg13Op::Addr as u16) | devad;
        self.smi_write(phy_addr, C22::MMD_CONTROL, val)?;
        // Write Addr
        self.smi_write(phy_addr, C22::MMD_DATA, reg.regnum)?;

        // Write FN
        let val = (Reg13Op::Read as u16) | devad;
        self.smi_write(phy_addr, C22::MMD_CONTROL, val)?;
        // Write Addr
        self.smi_read(phy_addr, C22::MMD_DATA)
    }

    /// Write, Clause 45
    /// This is the default implementation.
    /// Many hardware these days support direct Clause 45 operations.
    /// Implement this function when your hardware supports it.
    fn smi_write_mmd(&mut self, phy_addr: u8, reg: C45, reg_val: u16) -> Result<(), Self::Error> {
        let devad = u16::from(reg.devad.0 & DEV_MASK);

        // Write FN
        let val = (Reg13Op::Addr as u16) | devad;
        self.smi_write(phy_addr, C22::MMD_CONTROL, val)?;
        // Write Addr
        self.smi_write(phy_addr, C22::MMD_DATA, reg.regnum)?;

        // Write FN
        let val = (Reg13Op::Write as u16) | devad;
        self.smi_write(phy_addr, C22::MMD_CONTROL, val)?;
        // Write Addr
        self.smi_write(phy_addr, C22::MMD_DATA, reg_val)
    }
}

/// Station Management Interface (SMI) on an ethernet PHY Async
pub trait StationManagementAsync {
    /// `StationManagement` error type
    type Error: core::fmt::Debug;

    /// Read a register over SMI.
    fn smi_read(&mut self, phy_addr: u8, reg: C22) -> impl Future<Output = Result<u16, Self::Error>>;
    /// Write a register over SMI.
    fn smi_write(&mut self, phy_addr: u8, reg: C22, val: u16) -> impl Future<Output = Result<(), Self::Error>>;

    /// Read, Clause 45
    /// This is the default implementation.
    /// Many hardware these days support direct Clause 45 operations.
    /// Implement this function when your hardware supports it.
    fn smi_read_mmd(&mut self, phy_addr: u8, reg: C45) -> impl Future<Output = Result<u16, Self::Error>> {
        async move {
            let devad = u16::from(reg.devad.0 & DEV_MASK);

            // Write FN
            let val = (Reg13Op::Addr as u16) | devad;
            self.smi_write(phy_addr, C22::MMD_CONTROL, val).await?;
            // Write Addr
            self.smi_write(phy_addr, C22::MMD_DATA, reg.regnum).await?;

            // Write FN
            let val = (Reg13Op::Read as u16) | devad;
            self.smi_write(phy_addr, C22::MMD_CONTROL, val).await?;
            // Write Addr
            self.smi_read(phy_addr, C22::MMD_DATA).await
        }
    }

    /// Write, Clause 45
    /// This is the default implementation.
    /// Many hardware these days support direct Clause 45 operations.
    /// Implement this function when your hardware supports it.
    fn smi_write_mmd(&mut self, phy_addr: u8, reg: C45, reg_val: u16) -> impl Future<Output = Result<(), Self::Error>> {
        async move {
            let devad = u16::from(reg.devad.0 & DEV_MASK);

            // Write FN
            let val = (Reg13Op::Addr as u16) | devad;
            self.smi_write(phy_addr, C22::MMD_CONTROL, val).await?;
            // Write Addr
            self.smi_write(phy_addr, C22::MMD_DATA, reg.regnum).await?;

            // Write FN
            let val = (Reg13Op::Write as u16) | devad;
            self.smi_write(phy_addr, C22::MMD_CONTROL, val).await?;
            // Write Addr
            self.smi_write(phy_addr, C22::MMD_DATA, reg_val).await
        }
    }
}

#[cfg(test)]
mod tests_sync {
    extern crate alloc;
    use alloc::{vec, vec::Vec};

    use core::convert::Infallible;

    use crate::{
        phy::regs::{Mmd, C45},
        StationManagement,
    };

    use super::C22;

    #[derive(Debug, PartialEq)]
    enum A {
        Read(u8, C22),
        Write(u8, C22, u16),
    }

    struct MockMdioBus(Vec<A>);

    impl MockMdioBus {
        pub fn clear(&mut self) {
            self.0.clear();
        }
    }

    impl StationManagement for MockMdioBus {
        type Error = Infallible;

        fn smi_read(&mut self, phy_addr: u8, reg: C22) -> Result<u16, Self::Error> {
            self.0.push(A::Read(phy_addr, reg));
            Ok(0)
        }

        fn smi_write(&mut self, phy_addr: u8, reg: C22, val: u16) -> Result<(), Self::Error> {
            self.0.push(A::Write(phy_addr, reg, val));
            Ok(())
        }
    }

    #[test]
    fn read_test() {
        let mut mdiobus = MockMdioBus(Vec::with_capacity(20));

        mdiobus.clear();
        assert_eq!(mdiobus.smi_read(0x01, C22(0x00)), Ok(0));
        assert_eq!(mdiobus.0, vec![A::Read(0x01, C22(0x00))]);

        mdiobus.clear();
        assert_eq!(mdiobus.smi_read_mmd(0x01, C45::new(Mmd(0xBB), 0x1234)), Ok(0));
        assert_eq!(
            mdiobus.0,
            vec![
                #[allow(clippy::identity_op)]
                A::Write(0x01, C22::MMD_CONTROL, (0b00 << 14) | 27),
                A::Write(0x01, C22::MMD_DATA, 0x1234),
                A::Write(0x01, C22::MMD_CONTROL, (0b11 << 14) | 27),
                A::Read(0x01, C22::MMD_DATA)
            ]
        );
    }

    #[test]
    fn write_test() {
        let mut mdiobus = MockMdioBus(Vec::with_capacity(20));

        mdiobus.clear();
        mdiobus.smi_write(0x1f, C22(0xDA), 0xBCDE).unwrap();
        assert_eq!(mdiobus.0, vec![A::Write(0x1f, C22(0xDA), 0xBCDE)]);

        mdiobus.clear();
        assert_eq!(mdiobus.smi_write_mmd(0x1f, C45::new(Mmd(0xBB), 0x3456), 0xCDEF), Ok(()));
        assert_eq!(
            mdiobus.0,
            vec![
                A::Write(0x1f, C22::MMD_CONTROL, 27),
                A::Write(0x1f, C22::MMD_DATA, 0x3456),
                A::Write(0x1f, C22::MMD_CONTROL, (0b01 << 14) | 27),
                A::Write(0x1f, C22::MMD_DATA, 0xCDEF)
            ]
        );
    }
}

#[cfg(test)]
mod tests_async {
    extern crate alloc;
    use alloc::{vec, vec::Vec};

    use core::convert::Infallible;

    use crate::{
        phy::regs::{Mmd, C45},
        StationManagementAsync,
    };

    use super::C22;

    #[derive(Debug, PartialEq)]
    enum A {
        Read(u8, C22),
        Write(u8, C22, u16),
    }

    struct MockMdioBus(Vec<A>);

    impl MockMdioBus {
        pub fn clear(&mut self) {
            self.0.clear();
        }
    }

    impl StationManagementAsync for MockMdioBus {
        type Error = Infallible;

        async fn smi_read(&mut self, phy_addr: u8, reg: C22) -> Result<u16, Self::Error> {
            self.0.push(A::Read(phy_addr, reg));
            Ok(0)
        }

        async fn smi_write(&mut self, phy_addr: u8, reg: C22, val: u16) -> Result<(), Self::Error> {
            self.0.push(A::Write(phy_addr, reg, val));
            Ok(())
        }
    }

    #[futures_test::test]
    async fn read_test() {
        let mut mdiobus = MockMdioBus(Vec::with_capacity(20));

        mdiobus.clear();
        assert_eq!(mdiobus.smi_read(0x01, C22(0x00)).await, Ok(0));
        assert_eq!(mdiobus.0, vec![A::Read(0x01, C22(0x00))]);

        mdiobus.clear();
        assert_eq!(mdiobus.smi_read_mmd(0x01, C45::new(Mmd(0xBB), 0x1234)).await, Ok(0));
        assert_eq!(
            mdiobus.0,
            vec![
                #[allow(clippy::identity_op)]
                A::Write(0x01, C22::MMD_CONTROL, (0b00 << 14) | 27),
                A::Write(0x01, C22::MMD_DATA, 0x1234),
                A::Write(0x01, C22::MMD_CONTROL, (0b11 << 14) | 27),
                A::Read(0x01, C22::MMD_DATA)
            ]
        );
    }

    #[futures_test::test]
    async fn write_test() {
        let mut mdiobus = MockMdioBus(Vec::with_capacity(20));

        mdiobus.clear();
        mdiobus.smi_write(0x1f, C22(0xAA), 0xABCD).await.unwrap();
        assert_eq!(mdiobus.0, vec![A::Write(0x1f, C22(0xAA), 0xABCD)]);

        mdiobus.clear();
        assert_eq!(
            mdiobus.smi_write_mmd(0x1f, C45::new(Mmd(0xBB), 0x1234), 0xABCD).await,
            Ok(())
        );
        assert_eq!(
            mdiobus.0,
            vec![
                A::Write(0x1f, C22::MMD_CONTROL, 27),
                A::Write(0x1f, C22::MMD_DATA, 0x1234),
                A::Write(0x1f, C22::MMD_CONTROL, (0b01 << 14) | 27),
                A::Write(0x1f, C22::MMD_DATA, 0xABCD)
            ]
        );
    }
}
