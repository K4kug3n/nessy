use nessy::cartridge::Cartridge;
use nessy::mapper::Mapper;
use nessy::cpu::{Cpu, trace};
use nessy::memory::Memory;

use std::io::prelude::*;
use std::fs::File;

fn main() {
    let mut file = File::open("rom/nestest.nes").expect("Could not read the file {}");
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Could not read bytes");

    let cartridge = Cartridge::from_ines(&buffer);
    let mapper = <dyn Mapper>::from_id(cartridge.mapper, cartridge.pgr_rom.clone(), cartridge.chr_rom.clone());
    let mut memory = Memory::new(mapper);

    let mut cpu = Cpu::new();
    cpu.reset(&mut memory);
    cpu.pc = 0xC000;

    cpu.run_with_callback(&mut memory, |cpu, memory| {
        println!("{}", trace(cpu, memory));
    });
}
