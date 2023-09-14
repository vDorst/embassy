//! This example shows how to use SPI (Serial Peripheral Interface) in the RP2040 chip.
//!
//! Example written for a display using the ST7789 chip. Possibly the Waveshare Pico-ResTouch
//! (https://www.waveshare.com/wiki/Pico-ResTouch-LCD-2.8)

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(unused_imports)]

use core::cell::RefCell;

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_lora::iv::GenericSx126xInterfaceVariant;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{self, Async, Blocking, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use embedded_hal_async::spi::SpiBus;
use lora_phy::mod_params::*;
use lora_phy::sx1261_2::SX1261_2;
use lora_phy::LoRa;
use st7789::{Orientation, ST7789};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const LORA_FREQUENCY_IN_HZ: u32 = 868_500_000; // warning: set this appropriately for the region

use crate::my_display_interface::SPIDeviceInterface;

const DISPLAY_FREQ: u32 = 64_000_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<'static, SPI1, Async>>> = StaticCell::new();

    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let bl = p.PIN_25;
    let rst = p.PIN_13;
    let display_cs = p.PIN_9;
    let dcx = p.PIN_8;
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;

    // create SPI
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    let spi = Spi::new(p.SPI1, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());

    // Commit these two lines below to make it compile right with lora.
    let spi_bus = Mutex::<NoopRawMutex, _>::new(spi);
    let spi = SPI_BUS.init(spi_bus);

    //let display_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(display_cs, Level::High), display_config);

    // let dcx = Output::new(dcx, Level::Low);
    // let rst = Output::new(rst, Level::Low);
    // // dcx: 0 = command, 1 = data

    // // Enable LCD backlight
    // let _bl = Output::new(bl, Level::High);

    // // display interface abstraction from SPI and DC
    // let di = SPIDeviceInterface::new(display_spi, dcx);

    // // create driver
    // let mut display = ST7789::new(di, rst, 240, 320);

    let mut delay = Delay;

    // // initialize
    // display.init(&mut delay).unwrap();

    // // set default orientation
    // display.set_orientation(Orientation::Landscape).unwrap();

    // display.clear(Rgb565::BLACK).unwrap();

    // let raw_image_data = ImageRawLE::new(include_bytes!("../../assets/ferris.raw"), 86);
    // let ferris = Image::new(&raw_image_data, Point::new(34, 68));

    // // Display the image
    // ferris.draw(&mut display).unwrap();

    // let style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    // Text::new(
    //     "Hello embedded_graphics \n + embassy + RP2040!",
    //     Point::new(20, 30),
    //     style,
    // )
    // .draw(&mut display)
    // .unwrap();

    let nss = Output::new(p.PIN_3.degrade(), Level::High);
    let reset = Output::new(p.PIN_15.degrade(), Level::High);
    let dio1 = Input::new(p.PIN_20.degrade(), Pull::None);
    let busy = Input::new(p.PIN_2.degrade(), Pull::None);

    let iv = GenericSx126xInterfaceVariant::new(nss, reset, dio1, busy, None, None).unwrap();

    let mut lora = {
        match LoRa::new(
            SX1261_2::new(BoardType::RpPicoWaveshareSx1262, spi, iv),
            false,
            &mut delay,
        )
        .await
        {
            Ok(l) => l,
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        }
    };

    let mut debug_indicator = Output::new(p.PIN_24, Level::Low);

    let mut receiving_buffer = [00u8; 100];

    let mdltn_params = {
        match lora.create_modulation_params(
            SpreadingFactor::_10,
            Bandwidth::_250KHz,
            CodingRate::_4_8,
            LORA_FREQUENCY_IN_HZ,
        ) {
            Ok(mp) => mp,
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        }
    };

    let rx_pkt_params = {
        match lora.create_rx_packet_params(4, false, receiving_buffer.len() as u8, true, false, &mdltn_params) {
            Ok(pp) => pp,
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        }
    };

    match lora
        .prepare_for_rx(&mdltn_params, &rx_pkt_params, None, true, false, 0, 0x00ffffffu32)
        .await
    {
        Ok(()) => {}
        Err(err) => {
            info!("Radio error = {}", err);
            return;
        }
    };

    loop {
        receiving_buffer = [00u8; 100];
        match lora.rx(&rx_pkt_params, &mut receiving_buffer).await {
            Ok((received_len, _rx_pkt_status)) => {
                if (received_len == 3)
                    && (receiving_buffer[0] == 0x01u8)
                    && (receiving_buffer[1] == 0x02u8)
                    && (receiving_buffer[2] == 0x03u8)
                {
                    info!("rx successful");
                    debug_indicator.set_high();
                    Timer::after(Duration::from_secs(5)).await;
                    debug_indicator.set_low();
                } else {
                    info!("rx unknown packet");
                }
            }
            Err(err) => info!("rx unsuccessful = {}", err),
        }
    }
}

mod my_display_interface {
    use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
    use embedded_hal_1::digital::OutputPin;
    use embedded_hal_1::spi::SpiDevice;

    /// SPI display interface.
    ///
    /// This combines the SPI peripheral and a data/command pin
    pub struct SPIDeviceInterface<SPI, DC> {
        spi: SPI,
        dc: DC,
    }

    impl<SPI, DC> SPIDeviceInterface<SPI, DC>
    where
        SPI: SpiDevice,
        DC: OutputPin,
    {
        /// Create new SPI interface for communciation with a display driver
        pub fn new(spi: SPI, dc: DC) -> Self {
            Self { spi, dc }
        }
    }

    impl<SPI, DC> WriteOnlyDataCommand for SPIDeviceInterface<SPI, DC>
    where
        SPI: SpiDevice,
        DC: OutputPin,
    {
        fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
            // 1 = data, 0 = command
            self.dc.set_low().map_err(|_| DisplayError::DCError)?;

            send_u8(&mut self.spi, cmds).map_err(|_| DisplayError::BusWriteError)?;
            Ok(())
        }

        fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
            // 1 = data, 0 = command
            self.dc.set_high().map_err(|_| DisplayError::DCError)?;

            send_u8(&mut self.spi, buf).map_err(|_| DisplayError::BusWriteError)?;
            Ok(())
        }
    }

    fn send_u8<T: SpiDevice>(spi: &mut T, words: DataFormat<'_>) -> Result<(), T::Error> {
        match words {
            DataFormat::U8(slice) => spi.write(slice),
            DataFormat::U16(slice) => {
                use byte_slice_cast::*;
                spi.write(slice.as_byte_slice())
            }
            DataFormat::U16LE(slice) => {
                use byte_slice_cast::*;
                for v in slice.as_mut() {
                    *v = v.to_le();
                }
                spi.write(slice.as_byte_slice())
            }
            DataFormat::U16BE(slice) => {
                use byte_slice_cast::*;
                for v in slice.as_mut() {
                    *v = v.to_be();
                }
                spi.write(slice.as_byte_slice())
            }
            DataFormat::U8Iter(iter) => {
                let mut buf = [0; 32];
                let mut i = 0;

                for v in iter.into_iter() {
                    buf[i] = v;
                    i += 1;

                    if i == buf.len() {
                        spi.write(&buf)?;
                        i = 0;
                    }
                }

                if i > 0 {
                    spi.write(&buf[..i])?;
                }

                Ok(())
            }
            DataFormat::U16LEIter(iter) => {
                use byte_slice_cast::*;
                let mut buf = [0; 32];
                let mut i = 0;

                for v in iter.map(u16::to_le) {
                    buf[i] = v;
                    i += 1;

                    if i == buf.len() {
                        spi.write(&buf.as_byte_slice())?;
                        i = 0;
                    }
                }

                if i > 0 {
                    spi.write(&buf[..i].as_byte_slice())?;
                }

                Ok(())
            }
            DataFormat::U16BEIter(iter) => {
                use byte_slice_cast::*;
                let mut buf = [0; 64];
                let mut i = 0;
                let len = buf.len();

                for v in iter.map(u16::to_be) {
                    buf[i] = v;
                    i += 1;

                    if i == len {
                        spi.write(&buf.as_byte_slice())?;
                        i = 0;
                    }
                }

                if i > 0 {
                    spi.write(&buf[..i].as_byte_slice())?;
                }

                Ok(())
            }
            _ => unimplemented!(),
        }
    }
}
