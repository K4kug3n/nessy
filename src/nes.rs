use crate::cpu::Cpu;
use crate::memory::Memory;

pub struct Nes {
	cpu: Cpu,
	memory:  Memory
}

impl Nes {
	pub fn new() -> Nes {
		Nes {
			cpu: Cpu::new(),
			memory: Memory::new()

		}
	}

	pub fn play(&mut self) {
		self.cpu.step(&mut self.memory);
	}
}
