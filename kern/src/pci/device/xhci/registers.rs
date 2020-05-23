use volatile::{Volatile, WriteOnly};

#[repr(C)]
pub struct InterrupterRegisters {
    /// Interrupt Enable | Int Pending
    pub flags: Volatile<u32>,
    pub moderation_interval: Volatile<u16>,
    pub moderation_counter: Volatile<u16>,
    pub event_ring_table_size: Volatile<u32>,
    _res3: u32,
    pub event_ring_seg_table_ptr: Volatile<u64>,
    ///  Busy(3) | (2:0)index
    pub event_ring_deque_ptr: Volatile<u64>,
}

impl InterrupterRegisters {
    pub fn pending(&self) -> bool {
        self.flags.read() & 0x1 == 1
    }
}

#[repr(C)]
pub struct DoorBellRegister {
    pub reg: WriteOnly<u32>,
}