use self::device::PCIDevice;
use self::class::{PCIDeviceClass, HeaderType, PCISerialBusControllerClass, PCISerialBusUSB};
use alloc::vec::Vec;
use alloc::alloc::handle_alloc_error;
use spin::Mutex;
use crate::device::ahci::{AHCI, G_AHCI};
use crate::device::usb::G_USB;
use crate::device::uart::serial16650::Serial16650;

pub mod device;
pub mod class;
pub mod consts;

pub static GLOBAL_PCI: Mutex<PCIController> = Mutex::new(PCIController {
    devices: None,
    max_bus_num: 0
});

pub struct PCIController {
    devices: Option<Vec<PCIDevice>>,
    max_bus_num: u8,
}

#[derive(Debug, Clone)]
pub enum PCIError {
    SlotNumber,
    FuncNumber,
    RegisterNumber,
    InvalidDevice,
}

#[derive(Debug, Clone)]
pub enum PCICapabilityID {
    PowerManagement,
    AGP,
    VPD,
    SlotID,
    MSI,
    CompactPCIHotSwap,
    PCIX,
    HyperTrasnport,
    Vendor,
    Debug,
    CompactPCICentralResourceCtrl,
    PCIHotPlug,
    PCIBridgeSubsystemVendorID,
    AGP8x,
    SecureDevice,
    PCIExpress,
    MSIX,
    Unknown(u8),
}

impl From<u8> for PCICapabilityID {
    fn from(id: u8) -> Self {
        use PCICapabilityID::*;
        match id {
            0x1 => PowerManagement,
            0x2 => AGP,
            0x3 => VPD,
            0x4 => SlotID,
            0x5 => MSI,
            0x6 => CompactPCIHotSwap,
            0x7 => PCIX,
            0x8 => HyperTrasnport,
            0x9 => Vendor,
            0xa => Debug,
            0xb => CompactPCICentralResourceCtrl,
            0xc => PCIHotPlug,
            0xd => PCIBridgeSubsystemVendorID,
            0xe => AGP8x,
            0xf => SecureDevice,
            0x10 => PCIExpress,
            0x11 => MSIX,
            _ => Unknown(id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PCICapability {
    pub id: PCICapabilityID,
    /// Byte based address
    pub addr: u8,
}

impl PCICapability {
    pub fn new(id: u8, addr: u8) -> PCICapability {
        PCICapability {
            id: PCICapabilityID::from(id),
            addr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PCIDeviceInfo {
    pub class: PCIDeviceClass,
    pub rev: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub header_type: HeaderType,
    pub capabilities: Vec<PCICapability>,
}

impl PCIDeviceInfo {
    pub fn new(class_code: u32, rev: u8, vendor_id: u16, device_id: u16, header_type: u8) -> PCIDeviceInfo {
        let class = PCIDeviceClass::from(class_code);
        PCIDeviceInfo {
            class,
            rev,
            vendor_id,
            device_id,
            header_type: HeaderType::from(header_type),
            capabilities: Vec::default(),
        }
    }
}

impl PCIController {
    pub fn initialize_bus_with_devices(&mut self) {
        use self::class::*;
        info!("[PCI] Initializing PCI Devices");
        self.scan_pci_bus();
        let bus = self.enumerate_pci_bus();
        for dev in bus.into_iter() {
            match dev.info.class {
                PCIDeviceClass::MassStorageController(mass_storage) => {
                    match mass_storage {
                        PCIClassMassStorageClass::SATA(sata) => {
                            match sata {
                                PCIClassMassStroageSATA::AHCI => {
                                    G_AHCI.initialize_device(dev)
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                PCIDeviceClass::SimpleCommunicationController(comm) => {
                    match comm {
                        PCISimpleCommunicationControllerClass::SerialController(_) => {
                            crate::device::uart::serial16650::pci_load_16650_serial(dev);
                        },
                        _ => {}
                    }
                },
                PCIDeviceClass::SerialBusController(serialbus) => {
                    match serialbus {
                        PCISerialBusControllerClass::USBController(usb_ctlr) => {
                            debug!("USB on {} -> {:04x}:{:04x}", dev.bus_location_str(), dev.info.vendor_id, dev.info.device_id);
                            match usb_ctlr {
                                PCISerialBusUSB::XHCI => {
                                    crate::device::usb::xhci::load_from_device(dev);
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        info!("[PCI] Initialization Complete");
    }

    pub fn enumerate_pci_bus(&self) -> Vec<PCIDevice> {
        if let Some(dev) = self.devices.as_ref() {
            return dev.clone()
        }
        panic!("PCI Bus is never scanned");
    }

    pub fn scan_pci_bus(&mut self) {
        let mut bus = Vec::<PCIDevice>::with_capacity(16);
        self.max_bus_num = 0;
        self.enumerate_bus(0, &mut bus);
        self.devices = Some(bus);
    }

    fn enumerate_bus(&mut self, bus: u8, vec: &mut Vec<PCIDevice>) {
        if bus > self.max_bus_num {
            self.max_bus_num = bus;
        }
        for device_id in 0..32 {
            self.check_device(bus, device_id, vec);
        }
    }

    fn check_device(&mut self, bus: u8, device: u8, vec: &mut Vec<PCIDevice>) {
        if self.check_function(bus, device, 0, vec) {
            for func in 1..8 {
                self.check_function(bus, device, func, vec);
            }
        }
    }

    /// Returns: isMultiFunction
    fn check_function(&mut self, bus: u8, device: u8, func: u8, vec: &mut Vec<PCIDevice>) -> bool {
        let dev = PCIDevice::new(bus, device, func);
        match dev {
            Some(mut d) => {
                let mf = d.info.header_type.is_multi_function();
                if let HeaderType::PCIBridge(_) = d.info.header_type {
                    let mut num = d.secondary_bus_number();
                    if num == 0 {
                        self.max_bus_num+=1;
                        num = self.max_bus_num;
                        d.set_secondary_bus_number(num);
                        assert_ne!(d.secondary_bus_number(), 0);
                    }
                    trace!("PCI Bridge to : {}", num);
                    if num != bus {
                        self.enumerate_bus(num, vec);
                    } else {
                        error!("PCI Bridge to self? on {}", bus);
                    }
                }
                vec.push(d);
                mf
            }
            None => false
        }
    }
}

