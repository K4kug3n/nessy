use crate::{rom::Rom, ppu::Ppu};

const RAM: u16 = 0x0000;
const RAM_MIRROR_END: u16 = 0x1FFF;
const PPU_MIRROR: u16 = 0x2008;
const PPU_MIRROR_END: u16 = 0x3FFF;
const CARTRIDGE: u16 = 0x4020;
const CARTRIDGE_END: u16 = 0xFFFF;

pub struct Bus {
	cpu_ram: [u8; 2048],
	rom: Rom,
	ppu: Ppu
}

impl Bus {
	pub fn new(rom: Rom) -> Bus {
		let ppu = Ppu::new(rom.mirroring);
		Bus {
			cpu_ram: [0; 2048],
			rom,
			ppu
		}
	}

	pub fn read(&mut self, adress: u16) -> u8 {
		match adress {
			RAM..=RAM_MIRROR_END => {
				self.cpu_ram[usize::from(adress & 0x07FF)]
			},
			0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", adress);
            }
            0x2007 => self.ppu.read(&self.rom),
			PPU_MIRROR..=PPU_MIRROR_END => {
				let mirror_down_addr = adress & 0x2007;
                self.read(mirror_down_addr)
			},
			CARTRIDGE..=CARTRIDGE_END => {
				self.rom.mapper.read(adress)
			},
			_ => panic!("{} not adressed in cpu", adress)
		}
		
	}

	pub fn read_u16(&mut self, adress: u16) -> u16 {
		let low = self.read(adress) as u16;
		let high = self.read(adress + 1) as u16;
		
		(high << 8) | low
	}

	pub fn write(&mut self, adress: u16, value: u8) {
		match adress {
			RAM..=RAM_MIRROR_END => {
				self.cpu_ram[usize::from(adress & 0x07FF)] = value;
			},
			0x2000 => self.ppu.ctrl.write(value),
            0x2006 => self.ppu.addr.write(value),
            0x2007 => self.ppu.write(value),
			PPU_MIRROR..=PPU_MIRROR_END => {
				let mirror_down_addr = adress & 0x2007;
                self.write(mirror_down_addr, value);
			},
			CARTRIDGE..=CARTRIDGE_END => {
				self.rom.mapper.write(adress, value);
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

	pub fn read_chr_rom(&self, adress: u16) -> u8 {
		self.rom.mapper.read_chr_rom(adress)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::rom::test;

	#[test]
	fn cpu_write_and_read() {
		let mut bus = Bus::new(test::test_rom());

		bus.write(0x06e2, 0x25);
		assert_eq!(bus.read(0x06e2), 0x25);
		bus.write(0x06e3, 0x10);
		assert_eq!(bus.read(0x06e3), 0x10);
		bus.write(0x06e1, 0x07);
		assert_eq!(bus.read(0x06e1), 0x07);

		assert_eq!(bus.read(0x06e2), 0x25);
	}

	#[test]
	fn cpu_mirroring() {
		let mut bus = Bus::new(test::test_rom());

		bus.write(0x0000, 0x17);
		assert_eq!(bus.read(0x0800), 0x17);
		assert_eq!(bus.read(0x1000), 0x17);
		assert_eq!(bus.read(0x1800), 0x17);

		bus.write(0x0820, 0x07);
		assert_eq!(bus.read(0x0020), 0x07);
		assert_eq!(bus.read(0x1020), 0x07);
		assert_eq!(bus.read(0x1820), 0x07);
	}
}