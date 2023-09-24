use crate::cpu::Cpu;
use crate::memory::Memory;
use crate::cartridge::Cartridge;
use crate::mapper::Mapper;
use crate::ppu::Ppu;

pub struct Nes {
	cpu: Cpu,
	ppu: Ppu,
	memory:  Memory,
}

impl Nes {
	pub fn new(cartridge: &Cartridge) -> Nes {
		let mapper = <dyn Mapper>::from_id(cartridge.mapper, cartridge.pgr_rom.clone(), cartridge.chr_rom.clone());

		Nes {
			cpu: Cpu::new(),
			ppu: Ppu::new(cartridge.mirroring),
			memory: Memory::new(mapper),
		}
	}

	pub fn play(&mut self) {
		self.cpu.step(&mut self.memory);
	}
}
