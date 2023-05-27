mod cpu_ram;
mod palettes;
mod registers;
mod vram;

use crossbeam::channel::{bounded, select, Receiver, Sender};


pub struct RWMessage {
    pub operate_type: RWType,
    pub address: u16,
    pub value: Option<u8>,
}

pub enum RWType {
    Read,
    Write,
}

pub struct RWResult {
    pub data: Option<u8>,
    pub is_success: bool,
}

pub struct InterruptVectors {
    pub nmi_vector: u16,
    pub reset_vector: u16,
    pub irq_vector: u16,
}

#[derive(Clone)]
pub struct CpuBus {
    cpu2bus_out: Receiver<RWMessage>,
    pub cpu2bus_in: Sender<RWMessage>,
    bus2cpu_in: Sender<RWResult>,
    pub bus2cpu_out: Receiver<RWResult>,
}

#[derive(Clone)]
pub struct PpuBus {
    ppu2bus_out: Receiver<RWMessage>,
    pub ppu2bus_in: Sender<RWMessage>,
    bus2ppu_in: Sender<RWResult>,
    pub bus2ppu_out: Receiver<RWResult>,
}
