//! Mock version of rusb::DeviceHandle
use mockall::mock;
use rtlsdr_rs::error::Result;

use std::time::Duration;

mock! {
    #[derive(Debug)]
    pub DeviceHandle {
        pub fn open_by_index(index: usize) -> Result<Self>;
        pub fn open_by_serial(serial: &str) -> Result<Self>;
        pub fn claim_interface(&mut self, iface: u8) -> Result<()>;
        pub fn reset(&mut self) -> Result<()>;
        pub fn read_control(
            &self,
            request_type: u8,
            request: u8,
            value: u16,
            index: u16,
            buf: &mut [u8],
            timeout: Duration,
        ) -> Result<usize>;
        pub fn write_control(
            &self,
            request_type: u8,
            request: u8,
            value: u16,
            index: u16,
            buf: &[u8],
            timeout: Duration,
        ) -> Result<usize>;
        pub fn read_bulk(
            &self,
            endpoint: u8,
            buf: &mut [u8],
            timeout: Duration,
        ) -> Result<usize>;

    }
}
