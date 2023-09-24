use crate::cartridge::Mirroring;

pub struct Ppu {
	mirroring: Mirroring
}

impl Ppu {
	pub fn new(mirroring: Mirroring) -> Ppu {
		Ppu {
			mirroring
		}
	}
}