//! A `display-interface` [`WriteOnlyDataCommand`] implementation
//! that wraps a [`SpiBus`] directly and keeps CS asserted for the
//! entire duration of each send, avoiding per-chunk CS toggling.
//!
//! Also implements [`RowWriter`] for `mipidsi::Display` to allow
//! direct row-by-row pixel output without dyn-iterator overhead.

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::{digital::OutputPin, spi::SpiBus};
use embedded_hal_async::spi::SpiBus as AsyncSpiBus;
use mipidsi::{
    Display,
    dcs::{SetColumnAddress, SetPageAddress, WriteMemoryStart},
    models::Model,
};

use crate::playfield::RowWriter;

/// Trait for async data writes to a display interface.
pub trait AsyncDataWrite {
    async fn write_bytes_async(&mut self, data: &[u8]);
}

impl<DI, M, RST> RowWriter for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand + AsyncDataWrite,
    M: Model,
    RST: OutputPin,
{
    fn begin_frame(&mut self) {
        let w = M::FRAMEBUFFER_SIZE.0 - 1;
        let h = M::FRAMEBUFFER_SIZE.1 - 1;
        let dcs = unsafe { self.dcs() };
        dcs.write_command(SetColumnAddress::new(0, w)).ok();
        dcs.write_command(SetPageAddress::new(0, h)).ok();
        dcs.write_command(WriteMemoryStart).ok();
    }

    async fn write_row(&mut self, row_bytes: &[u8]) {
        let dcs = unsafe { self.dcs() };
        dcs.di.write_bytes_async(row_bytes).await;
    }
}

pub struct DirectInterface<SPI, DC, CS> {
    spi: SPI,
    dc: DC,
    cs: CS,
}

impl<SPI, DC, CS> DirectInterface<SPI, DC, CS>
where
    SPI: SpiBus,
    DC: OutputPin,
    CS: OutputPin,
{
    pub fn new(spi: SPI, dc: DC, cs: CS) -> Self {
        Self { spi, dc, cs }
    }
}

impl<SPI, DC, CS> AsyncDataWrite for DirectInterface<SPI, DC, CS>
where
    SPI: AsyncSpiBus,
    DC: OutputPin,
    CS: OutputPin,
{
    async fn write_bytes_async(&mut self, data: &[u8]) {
        self.dc.set_high().ok();
        self.cs.set_low().ok();
        self.spi.write(data).await.ok();
        self.cs.set_high().ok();
    }
}

impl<SPI, DC, CS> WriteOnlyDataCommand for DirectInterface<SPI, DC, CS>
where
    SPI: SpiBus,
    DC: OutputPin,
    CS: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;
        let r = send_bytes(&mut self.spi, cmds);
        self.cs.set_high().map_err(|_| DisplayError::CSError)?;
        r
    }

    fn send_data(&mut self, data: DataFormat<'_>) -> Result<(), DisplayError> {
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;
        let r = send_bytes(&mut self.spi, data);
        self.cs.set_high().map_err(|_| DisplayError::CSError)?;
        r
    }
}

fn send_bytes<SPI: SpiBus>(spi: &mut SPI, data: DataFormat<'_>) -> Result<(), DisplayError> {
    match data {
        DataFormat::U8(bytes) => spi.write(bytes).map_err(|_| DisplayError::BusWriteError),
        DataFormat::U8Iter(iter) => {
            let mut buf = [0u8; 64];
            let mut n = 0;
            for byte in iter {
                buf[n] = byte;
                n += 1;
                if n == buf.len() {
                    spi.write(&buf).map_err(|_| DisplayError::BusWriteError)?;
                    n = 0;
                }
            }
            if n > 0 {
                spi.write(&buf[..n])
                    .map_err(|_| DisplayError::BusWriteError)?;
            }
            Ok(())
        }
        DataFormat::U16BE(words) => {
            // Safety: u16 and [u8; 2] have the same layout; we borrow as bytes.
            let bytes = unsafe {
                core::slice::from_raw_parts(words.as_ptr() as *const u8, words.len() * 2)
            };
            spi.write(bytes).map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16BEIter(iter) => {
            let mut buf = [0u8; 512];
            let mut n = 0;
            for word in iter {
                let [hi, lo] = word.to_be_bytes();
                buf[n] = hi;
                buf[n + 1] = lo;
                n += 2;
                if n == buf.len() {
                    spi.write(&buf).map_err(|_| DisplayError::BusWriteError)?;
                    n = 0;
                }
            }
            if n > 0 {
                spi.write(&buf[..n])
                    .map_err(|_| DisplayError::BusWriteError)?;
            }
            Ok(())
        }
        _ => Err(DisplayError::DataFormatNotImplemented),
    }
}
