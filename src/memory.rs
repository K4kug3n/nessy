const RAM: u16 = 0x0000;
const RAM_MIRROR_END: u16 = 0x1FFF;
const PPU: u16 = 0x2000;
const PPU_MIRROR_END: u16 = 0x3FFF;

pub struct Memory {
	cpu_ram: [u8; 2048]
}

impl Memory {
	pub fn new() -> Memory {
		Memory {
			cpu_ram: [0; 2048]
		}
	}

	pub fn read(&self, adress: u16) -> u8 {
		match adress {
			RAM..=RAM_MIRROR_END => {
				self.cpu_ram[usize::from(adress & 0x07FF)]
			},
			PPU..=PPU_MIRROR_END => {
				panic!("PPU not implemented");
			},
			_ => panic!("Memory out of range")
		}
		
	}

	pub fn write(&mut self, adress: u16, value: u8) {
		match adress {
			RAM..=RAM_MIRROR_END => {
				self.cpu_ram[usize::from(adress & 0x07FF)] = value;
			},
			PPU..=PPU_MIRROR_END => {
				panic!("PPU not implemented");
			},
			_ => panic!("Memory out of range")
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn cpu_write_and_read() {
		let mut memory = Memory::new();

		memory.write(0x06e2, 0x25);
		assert_eq!(memory.read(0x06e2), 0x25);
		memory.write(0x06e3, 0x10);
		assert_eq!(memory.read(0x06e3), 0x10);
		memory.write(0x06e1, 0x07);
		assert_eq!(memory.read(0x06e1), 0x07);

		assert_eq!(memory.read(0x06e2), 0x25);
	}

	#[test]
	fn cpu_mirroring() {
		let mut memory = Memory::new();

		memory.write(0x0000, 0x17);
		assert_eq!(memory.read(0x0800), 0x17);
		assert_eq!(memory.read(0x1000), 0x17);
		assert_eq!(memory.read(0x1800), 0x17);

		memory.write(0x0820, 0x07);
		assert_eq!(memory.read(0x0020), 0x07);
		assert_eq!(memory.read(0x1020), 0x07);
		assert_eq!(memory.read(0x1820), 0x07);
	}
}