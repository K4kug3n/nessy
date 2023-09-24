pub mod nrom;

use nrom::Nrom;

pub trait Mapper {
	fn cpu_read(&self, adress: u16) -> u8;
	fn cpu_write(&mut self, adress: u16, value: u8);

	fn ppu_read(&self, adress: u16) -> u8;
	fn ppu_write(&mut self, adress: u16, value: u8);
}

impl dyn Mapper {
	pub fn from_id(id: u8, pgr_rom: Vec<u8>, chr_rom: Vec<u8>) -> Box<dyn Mapper> {
		match id {
			0x0 => Box::new(Nrom::new(pgr_rom, chr_rom)),
			_ => panic!("Mapper {} not implemented", id)
		}
	}
}