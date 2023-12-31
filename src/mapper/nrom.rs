use crate::mapper::Mapper;

enum Variant {
	Nrom128,
	Nrom256
}

pub struct Nrom {
	variant: Variant,
	pgr_rom: Vec<u8>,
	chr_rom: Vec<u8>
}

impl Mapper for Nrom {
	fn cpu_read(&self, adress: u16) -> u8 {
        match adress {
			0x8000..=0xFFFF => {
				let effective = match self.variant {
					Variant::Nrom128 => adress & 0x3FFF,
					Variant::Nrom256 => adress & 0x7FFF
				};
				self.pgr_rom[usize::from(effective)]
			}
			_ => panic!("Undefined read mapping for {:#06x}", adress)
		}
    }

	fn cpu_write(&mut self, adress: u16, value: u8) {
        match adress {
			0x8000..=0xFFFF => {
				let effective = match self.variant {
					Variant::Nrom128 => adress & 0x3FFF,
					Variant::Nrom256 => adress & 0x7FFF
				};
				self.pgr_rom[usize::from(effective)] = value;
			}
			_ => panic!("Undefined write mapping for {:#06x}", adress)
		}
    }

	fn ppu_read(&self, adress: u16) -> u8 {
		match adress {
			0x0000..=0x1FFF => {
				self.chr_rom[usize::from(adress)]
			},
			_ => panic!("Undefined read mapping for {:#06x}", adress)
		}
    }

	fn ppu_write(&mut self, adress: u16, value: u8) {
        match adress {
			0x0000..=0x1FFF => {
				self.chr_rom[usize::from(adress)] = value;
			}
			_ => panic!("Undefined write mapping for {:#06x}", adress)
		}
    }
}

impl Nrom {
	pub fn new(pgr_rom: Vec<u8>, chr_rom: Vec<u8>) -> Nrom {
		let variant = if chr_rom.len() > 8196 { Variant::Nrom256 } else { Variant::Nrom128 };
		Nrom {
			variant,
			pgr_rom,
			chr_rom
		}
	}
}