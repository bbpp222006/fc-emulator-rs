use std::thread;
use crossbeam::channel::{bounded, select, Receiver, Sender};

use crate::memory::{RWResult,RWMessage,InterruptVectors};

pub struct Emulator {
    pub cpu_channels: CpuChannels,
    pub mem_channels: MemChannels,
    // pub window: Window,
}

#[derive(Clone)]
pub struct CpuChannels {
    pub cpu2mem_out: Receiver<RWMessage>,
    cpu2mem_in: Sender<RWMessage>,
    pub mem2cpu_in: Sender<RWResult>,
    mem2cpu_out: Receiver<RWResult>,
    pub clock_in: Sender<u64>,
    clock_out: Receiver<u64>,
}

#[derive(Clone)]
pub struct MemChannels {
    bus2mem_out: Receiver<RWMessage>,
    pub bus2mem_in: Sender<RWMessage>,
    mem2bus_in: Sender<RWResult>,
    pub mem2bus_out: Receiver<RWResult>,
    pub rom2mem_in: Sender<Vec<u8>>,
    rom2mem_out: Receiver<Vec<u8>>,
}


pub fn start_bus_thread() -> (CpuChannels, MemChannels) {
    let (mem2cpu_in, mem2cpu_out) = bounded(1);
    let (cpu2mem_in, cpu2mem_out) = bounded(1);
    let (clock_in, clock_out) = bounded(1);
    let cpu_channels = CpuChannels {
        cpu2mem_out,
        cpu2mem_in,
        mem2cpu_in,
        mem2cpu_out,
        clock_in,
        clock_out,
    };
    
    let (bus2mem_in, bus2mem_out) = bounded(1);
    let (mem2bus_in, mem2bus_out) = bounded(1);
    let (rom2mem_in, rom2mem_out) = bounded(1);
    let mem_channels = MemChannels {
        bus2mem_out,
        bus2mem_in,
        mem2bus_in,
        mem2bus_out,
        rom2mem_in,
        rom2mem_out,
    };
    thread::spawn(move || {
        let mem_channels = mem_channels.clone();
        let cpu_channels = cpu_channels.clone();
        loop {
            select! {
                recv(cpu2mem_out) -> msg => {
                    // cpu与mem的交互
                    mem_channels.bus2mem_in.send(msg.unwrap()).unwrap();
                    let result_cpu = mem_channels.mem2bus_out.recv().unwrap();
                    cpu_channels.mem2cpu_in.send(result_cpu).unwrap();
                },
                
            }
        }
    });

    (cpu_channels, mem_channels)
}


