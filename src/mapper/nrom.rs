use crate::mapper::Mapper;

pub struct Nrom {
	pgr_rom: Vec<u8>,
	chr_rom: Vec<u8>
}

impl Mapper for Nrom {
	fn read(&self, adress: u16) -> u8 {
        todo!()
    }

	fn write(&mut self, adress: u16, value: u8) {
        todo!()
    }
}

impl Nrom {
	pub fn new(pgr_rom: Vec<u8>, chr_rom: Vec<u8>) -> Nrom {
		Nrom {
			pgr_rom,
			chr_rom
		}
	}
}