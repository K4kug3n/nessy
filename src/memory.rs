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

	pub fn read(&self, adress: u16) -> u8 {
		match adress {
			RAM..=RAM_MIRROR_END => {
				self.cpu_ram[usize::from(adress & 0x07FF)]
			},
			PPU..=PPU_MIRROR_END => {
				panic!("PPU not implemented");
			},
			CARTRIDGE..=CARTRIDGE_END => {
				self.mapper.read(adress)
			},
			_ => panic!("{} not adressed in cpu", adress)
		}
		
	}

	pub fn read_u16(&self, adress: u16) -> u16 {
		let low = self.read(adress) as u16;
		let high = self.read(adress + 1) as u16;
		
		(high << 8) | low
	}

	pub fn write(&mut self, adress: u16, value: u8) {
		match adress {
			RAM..=RAM_MIRROR_END => {
				self.cpu_ram[usize::from(adress & 0x07FF)] = value;
			},
			PPU..=PPU_MIRROR_END => {
				panic!("PPU not implemented");
			},
			CARTRIDGE..=CARTRIDGE_END => {
				self.mapper.write(adress, value);
			},
			_ => panic!("{} not adressed in cpu", adress)
		}
	}

	pub fn write_u16(&mut self, adress: u16, value: u16) {
		let low = (value & 0x00FF) as u8;
		let high = (value >> 8) as u8;
		
		self.write(adress, low);
		self.write(adress + 1, high);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::mapper::nrom::Nrom;

	#[test]
	fn cpu_write_and_read() {
		let mut memory = Memory::new(Box::new(Nrom::new(Vec::new(), Vec::new())));

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
		let mut memory = Memory::new(Box::new(Nrom::new(Vec::new(), Vec::new())));

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