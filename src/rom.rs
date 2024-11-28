use crate::mapper::Mapper;

pub struct Rom {
	pub mapper: Box<dyn Mapper>,
	pub mirroring: Mirroring
}

#[derive(Clone, Copy)]
pub enum Mirroring {
	Vertical,
	Horizontal,
	FourScreen
}

impl Rom {
	pub fn from_ines(buffer: &[u8]) -> Rom {
		if buffer[0..=3] != [0x4e, 0x45, 0x53, 0x1a] {
			panic!("Wrong constants")
		}

		let pgr_rom_size = usize::from(buffer[4]) * 16384;
		let chr_rom_size = usize::from(buffer[5]) * 8192;

		let flag_6 = buffer[6];
		//let battery = flag_6 & 0x02;
		let trainer = (flag_6 & 0x04) != 0;

		let mirroring = (flag_6 & 0x01) != 0;
		let four_screen = (flag_6 & 0x08) != 0;
		let screen_mirroring = match (four_screen, mirroring) {
			(true, _) => Mirroring::FourScreen,
			(false, true) => Mirroring::Vertical,
			(false, false) => Mirroring::Horizontal
		};

		let low_mapper = flag_6 & 0xf0;
		
		let flag_7 = buffer[7];
		//let vs_unisystem = flag_7 & 0x01;
		//let play_choice_10 = flag_7 & 0x2;
		let nes_2 = (flag_7 & 0x0c) != 0;

		if nes_2 {
			panic!("NES 2.0 cartridge not supported")
		}

		let high_mapper = if /* !nes_2 && */ buffer[12..=15] != [0x0, 0x0, 0x0, 0x0] { 0x0 } else { flag_7 & 0xf0 };
		let mapper_id = high_mapper + (low_mapper >> 4);

		let pgr_rom_idx = usize::from(if trainer { 512u16 + 16u16 } else { 16u16 });
		let chr_rom_idx = pgr_rom_idx + pgr_rom_size;

		Rom { 
			mapper: <dyn Mapper>::from_id(
				mapper_id,
				buffer[pgr_rom_idx..(pgr_rom_idx + pgr_rom_size)].to_vec(),
				buffer[chr_rom_idx..(chr_rom_idx + chr_rom_size)].to_vec()
			),
			mirroring: screen_mirroring
		}
	}
}

pub mod test {
	use super::*;
	use crate::mapper::test;
	
	pub fn test_rom() -> Rom {
		// Empty rom (Nrom)
		Rom {
			mapper: test::test_mapper(),
			mirroring: Mirroring::Vertical
		}
	}
}