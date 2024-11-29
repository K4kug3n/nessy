pub mod nrom;

use nrom::Nrom;

pub trait Mapper {
	fn read(&self, adress: u16) -> u8;
	fn write(&mut self, adress: u16, value: u8);

	fn read_chr_rom(&self, adress: u16) -> u8;
}

impl dyn Mapper {
	pub fn from_id(id: u8, pgr_rom: Vec<u8>, chr_rom: Vec<u8>) -> Box<dyn Mapper> {
		match id {
			0x0 => Box::new(Nrom::new(pgr_rom, chr_rom)),
			_ => panic!("Mapper {} not implemented", id)
		}
	}
}

pub mod test {
use super::*;

	pub fn test_mapper() -> Box<dyn Mapper> {
		// Empty Nrom
		Box::new(Nrom::new(vec![0; 16384*2], vec![0; 8192]))
	}
}