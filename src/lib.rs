//! # rtlsdr Library
//! Library for interfacing with an RTL-SDR device.

pub mod device;
pub mod error;
pub mod rtlsdr;
pub mod tuners;
#[macro_use]
extern crate log;

use core::fmt;
use std::{io::Read, time::Duration};

use device::Device;
use error::Result;
use rtlsdr::RtlSdr as Sdr;
use tokio::io::AsyncRead;

pub const DEFAULT_BUF_LENGTH: usize = 16 * 16384;

#[derive(Debug)]
pub enum TunerGain {
    Auto,
    Manual(i32),
}

// implement fmt::Display for TunerGain

impl fmt::Display for TunerGain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TunerGain::Auto => write!(f, "Auto"),
            TunerGain::Manual(gain) => write!(f, "Manual({})", gain),
        }
    }
}

impl From<i32> for TunerGain {
    fn from(gain: i32) -> Self {
        TunerGain::Manual(gain)
    }
}

#[derive(Debug)]
pub enum DirectSampleMode {
    Off,
    On,
    OnSwap, // Swap I and Q ADC, allowing to select between two inputs
}

pub struct RtlSdr {
    sdr: Sdr,
}

impl Read for RtlSdr {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.read_sync(buf) {
            Ok(len) => Ok(len),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error reading from device: {:?}", e),
            )),
        }
    }
}

impl AsyncRead for RtlSdr {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context,
        buf: &mut tokio::io::ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut buffer = vec![0; buf.remaining()];
        match self.read_sync(&mut buffer) {
            Ok(len) => {
                buf.put_slice(&buffer[..len]);
                std::task::Poll::Ready(Ok(()))
            }
            Err(e) => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error reading from device: {:?}", e),
            ))),
        }
    }
}

impl RtlSdr {
    pub fn open_by_index(index: usize) -> Result<RtlSdr> {
        let dev = Device::new_by_index(index)?;
        let mut sdr = Sdr::new(dev);
        sdr.init()?;
        Ok(RtlSdr { sdr: sdr })
    }

    pub fn open_by_serial(serial: &str) -> Result<RtlSdr> {
        let dev = Device::new_by_serial(serial)?;
        let mut sdr = Sdr::new(dev);
        sdr.init()?;
        Ok(RtlSdr { sdr: sdr })
    }

    pub fn list_and_print_known_devices() -> Result<()> {
        Device::list_and_print_known_devices()
    }

    pub fn close(&mut self) -> Result<()> {
        // TODO: wait until async is inactive
        Ok(self.sdr.deinit_baseband()?)
    }
    pub fn reset_buffer(&self) -> Result<()> {
        self.sdr.reset_buffer()
    }
    pub fn read_sync(&self, buf: &mut [u8]) -> Result<usize> {
        self.sdr.read_sync(buf)
    }
    pub fn get_center_freq(&self) -> u32 {
        self.sdr.get_center_freq()
    }
    pub fn set_center_freq(&mut self, freq: u32) -> Result<()> {
        self.sdr.set_center_freq(freq)
    }
    pub fn get_tuner_gains(&self) -> Result<Vec<i32>> {
        self.sdr.get_tuner_gains()
    }
    pub fn set_tuner_gain(&mut self, gain: TunerGain) -> Result<()> {
        self.sdr.set_tuner_gain(gain)
    }
    pub fn get_freq_correction(&self) -> i32 {
        self.sdr.get_freq_correction()
    }
    pub fn set_freq_correction(&mut self, ppm: i32) -> Result<()> {
        self.sdr.set_freq_correction(ppm)
    }
    pub fn get_sample_rate(&self) -> u32 {
        self.sdr.get_sample_rate()
    }
    pub fn set_sample_rate(&mut self, rate: u32) -> Result<()> {
        self.sdr.set_sample_rate(rate)
    }
    pub fn set_tuner_bandwidth(&mut self, bw: u32) -> Result<()> {
        self.sdr.set_tuner_bandwidth(bw)
    }
    pub fn set_testmode(&mut self, on: bool) -> Result<()> {
        self.sdr.set_testmode(on)
    }
    pub fn set_direct_sampling(&mut self, mode: DirectSampleMode) -> Result<()> {
        self.sdr.set_direct_sampling(mode)
    }
    pub fn set_bias_tee(&self, on: bool) -> Result<()> {
        self.sdr.set_bias_tee(on)
    }
}
