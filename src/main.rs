use nessy::rom::Rom;
use nessy::cpu::{Cpu, trace};
use nessy::bus::Bus;

use std::io::prelude::*;
use std::fs::File;

fn main() {
    let mut file = File::open("rom/nestest.nes").expect("Could not read the file {}");
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Could not read bytes");

    let rom = Rom::from_ines(&buffer);
    let mut bus = Bus::new(rom);

    let mut cpu = Cpu::new();
    cpu.reset(&mut bus);
    cpu.pc = 0xC000;

    cpu.run_with_callback(&mut bus, |cpu: &mut Cpu, bus: &mut Bus| {
        println!("{}", trace(cpu, bus));
    });
}
