use crate::mapper::Mapper;

const RAM: u16 = 0x0000;
const RAM_MIRROR_END: u16 = 0x1FFF;
const PPU: u16 = 0x2000;
const PPU_MIRROR_END: u16 = 0x3FFF;
const CARTRIDGE: u16 = 0x4020;
const CARTRIDGE_END: u16 = 0xFFFF;

pub struct Memory {
	cpu_ram: [u8; 2048],
	mapper: Box<dyn Mapper>
}

impl Memory {
	pub fn new(mapper: Box<dyn Mapper>) -> Memory {
		Memory {
			cpu_ram: [0; 2048],
			mapper
		}
	}

	pub fn cpu_read(&self, adress: u16) -> u8 {
		match adress {
			RAM..=RAM_MIRROR_END => {
				self.cpu_ram[usize::from(adress & 0x07FF)]
			},
			PPU..=PPU_MIRROR_END => {
				panic!("PPU not implemented");
			},
			CARTRIDGE..=CARTRIDGE_END => {
				self.mapper.cpu_read(adress)
			},
			_ => panic!("{} not adressed in cpu", adress)
		}
		
	}

	pub fn cpu_write(&mut self, adress: u16, value: u8) {
		match adress {
			RAM..=RAM_MIRROR_END => {
				self.cpu_ram[usize::from(adress & 0x07FF)] = value;
			},
			PPU..=PPU_MIRROR_END => {
				panic!("PPU not implemented");
			},
			CARTRIDGE..=CARTRIDGE_END => {
				self.mapper.cpu_write(adress, value);
			},
			_ => panic!("{} not adressed in cpu", adress)
		}
	}

	pub fn ppu_read(&mut self, adress: u16) -> u8 {
		match adress {
			_ => panic!("{} not adressed in ppu", adress)
		}
	}

	pub fn ppu_write(&mut self, adress: u16, value: u8) {
		match adress {
			_ => panic!("{} not adressed in ppu", adress)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::mapper::nrom::Nrom;

	#[test]
	fn cpu_write_and_read() {
		let mut memory = Memory::new(Box::new(Nrom::new(Vec::new(), Vec::new())));

		memory.cpu_write(0x06e2, 0x25);
		assert_eq!(memory.cpu_read(0x06e2), 0x25);
		memory.cpu_write(0x06e3, 0x10);
		assert_eq!(memory.cpu_read(0x06e3), 0x10);
		memory.cpu_write(0x06e1, 0x07);
		assert_eq!(memory.cpu_read(0x06e1), 0x07);

		assert_eq!(memory.cpu_read(0x06e2), 0x25);
	}

	#[test]
	fn cpu_mirroring() {
		let mut memory = Memory::new(Box::new(Nrom::new(Vec::new(), Vec::new())));

		memory.cpu_write(0x0000, 0x17);
		assert_eq!(memory.cpu_read(0x0800), 0x17);
		assert_eq!(memory.cpu_read(0x1000), 0x17);
		assert_eq!(memory.cpu_read(0x1800), 0x17);

		memory.cpu_write(0x0820, 0x07);
		assert_eq!(memory.cpu_read(0x0020), 0x07);
		assert_eq!(memory.cpu_read(0x1020), 0x07);
		assert_eq!(memory.cpu_read(0x1820), 0x07);
	}
}