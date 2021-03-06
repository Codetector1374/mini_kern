use spin::{RwLock, Mutex};
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::sync::Arc;
use hashbrown::HashMap;
use core::sync::atomic::{AtomicU64, Ordering, AtomicBool};
use crate::device::uart::serial16650::Serial16650;

pub mod serial16650;

lazy_static! {
    pub static ref SERIAL_PORTS: RwLock<SerialPorts> = RwLock::new(SerialPorts::new());
}

pub static HAS_SERIAL: AtomicBool = AtomicBool::new(false);

pub trait UART : core_io::Write + core_io::Read + core::fmt::Write {
    fn set_baudrate(&mut self, baud: u32);
    fn write_byte(&mut self, byte: u8);
    fn read_byte(&mut self) -> Option<u8>;
    fn has_data(&self) -> bool;
}

pub struct SerialPorts {
    next_id: AtomicU64,
    pub ports: HashMap<u64, Arc<Mutex<dyn UART + Send + Sync>>>
}

impl SerialPorts {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            ports: HashMap::new()
        }
    }

    pub fn register_port(&mut self, port: Arc<Mutex<dyn UART + Send + Sync>>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::AcqRel);
        if id == 1 {
            self.ports.clear();
        }
        writeln!(port.lock(), "=============== PORT REGISTERED ======================").unwrap();
        self.ports.insert(id, port);
        HAS_SERIAL.store(true, Ordering::Relaxed);
        id
    }

    pub fn register_early_serial(&mut self, port: u16) -> bool {
        let mut port = Serial16650::new_from_port(port);
        if port.verify() {
            debug!("[Early Serial] Valid Serial Found");
            port.set_baudrate(115200);
            self.ports.insert(0, Arc::new(Mutex::new(port)));
            return true
        }
        false
    }
}
