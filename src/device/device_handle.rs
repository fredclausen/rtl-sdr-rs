use std::time::Duration;

use crate::error::Result;
use crate::error::RtlsdrError::RtlsdrErr;
use rusb::{Context, UsbContext};

use super::KNOWN_DEVICES;

#[derive(Debug)]
pub struct DeviceHandle {
    handle: rusb::DeviceHandle<Context>,
}

#[derive(Debug, Clone)]
pub struct KnownDevice<T: UsbContext> {
    pub name: String,
    pub device: rusb::Device<T>,
}

impl DeviceHandle {
    pub fn open(index: usize) -> Result<Self> {
        let mut context = Context::new()?;
        let handle = DeviceHandle::open_device(&mut context, index)?;
        Ok(DeviceHandle { handle: handle })
    }

    pub fn filter_known_devices<T: UsbContext>(context: &mut T) -> Result<Vec<KnownDevice<T>>> {
        let devices = context.devices().map(|d| d)?;

        let mut known_devices: Vec<KnownDevice<T>> = Vec::new();

        for device in devices.iter() {
            let device_desc = device.device_descriptor().map(|d| d)?;
            for dev in KNOWN_DEVICES.iter() {
                if device_desc.vendor_id() == dev.vid && device_desc.product_id() == dev.pid {
                    let known_device = KnownDevice {
                        name: dev.description.to_string(),
                        device: device.clone(),
                    };

                    known_devices.push(known_device);
                }
            }
        }

        Ok(known_devices)
    }

    pub fn print_known_devices<T: UsbContext>(devices: Vec<KnownDevice<T>>) {
        for dev in devices.iter() {
            let device_desc = dev.device.device_descriptor().unwrap();
            let name = dev.name.clone();
            println!(
                "Found device: Name: {} VID: {:04x} PID: {:04x}",
                name,
                device_desc.vendor_id(),
                device_desc.product_id());
        }
    }

    pub fn open_device<T: UsbContext>(
        context: &mut T,
        index: usize,
    ) -> Result<rusb::DeviceHandle<T>> {
        let devices = DeviceHandle::filter_known_devices(context)?;
        DeviceHandle::print_known_devices(devices.clone());

        // check and see if we have a device at index, and if so, return it
        if devices.len() > index {
            let device = devices.get(index).unwrap();
            let handle = device.device.open()?;
            return Ok(handle);
        }

        Err(RtlsdrErr(format!("No device found")))
    }

    pub fn claim_interface(&mut self, iface: u8) -> Result<()> {
        Ok(self.handle.claim_interface(iface)?)
    }
    pub fn reset(&mut self) -> Result<()> {
        Ok(self.handle.reset()?)
    }

    pub fn read_control(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize> {
        Ok(self
            .handle
            .read_control(request_type, request, value, index, buf, timeout)?)
    }

    pub fn write_control(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        buf: &[u8],
        timeout: Duration,
    ) -> Result<usize> {
        Ok(self
            .handle
            .write_control(request_type, request, value, index, buf, timeout)?)
    }

    pub fn read_bulk(&self, endpoint: u8, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        Ok(self.handle.read_bulk(endpoint, buf, timeout)?)
    }
}
