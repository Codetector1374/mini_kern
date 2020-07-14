use alloc::string::String;
use crate::device::usb::descriptor::USBDeviceDescriptor;

pub trait USBDevice {
    fn device_descriptor(&self) -> &USBDeviceDescriptor;

    fn manufacture_string(&self) -> &str;
    fn product_string(&self) -> &str;
    fn serial_string(&self) -> &str;

    fn bus(&self) -> u8;
    fn device(&self) -> u8;

    fn system_device_id(&self) -> u64;
}