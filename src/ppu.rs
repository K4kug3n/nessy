use crate::rom::{Mirroring, Rom};

pub struct AddrRegister {
	value: u16,
	is_hi: bool
}

impl AddrRegister {
	pub fn new() -> AddrRegister {
		AddrRegister {
			value: 0x00,
			is_hi: true
		}
	}

	pub fn write(&mut self, value: u8) {
		if self.is_hi {
			self.value = ((value as u16) << 8) | (self.value & 0x00FF);
		} else {
			self.value = (self.value & 0xFF00) | (value as u16);
		}
		if self.value > 0x3FFF {
			self.value = self.value & 0x3FFF; // Mirror down
		}

		self.is_hi = !self.is_hi;
	}

	pub fn increment(&mut self, value: u8) {
		self.value = self.value.wrapping_add(value as u16);

		if self.value > 0x3FFF {
			self.value = self.value & 0x3FFF; // Mirror down
		}
	}

	pub fn get(&self) -> u16 {
		self.value
	}
}

pub struct ControlRegister {
	// 7  bit  0
	// ---- ----
	// VPHB SINN
	// |||| ||||
	// |||| ||++- Base nametable address
	// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
	// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
	// |||| |     (0: add 1, going across; 1: add 32, going down)
	// |||| +---- Sprite pattern table address for 8x8 sprites
	// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
	// |||+------ Background pattern table address (0: $0000; 1: $1000)
	// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
	// |+-------- PPU master/slave select
	// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
	// +--------- Generate an NMI at the start of the
	//            vertical blanking interval (0: off; 1: on)
	value: u8
}

const NAMETABLE1             : u8 = 0b00000001;
const NAMETABLE2             : u8 = 0b00000010;
const VRAM_ADD_INCREMENT     : u8 = 0b00000100;
const SPRITE_PATTERN_ADDR    : u8 = 0b00001000;
const BACKROUND_PATTERN_ADDR : u8 = 0b00010000;
const SPRITE_SIZE            : u8 = 0b00100000;
const MASTER_SLAVE_SELECT    : u8 = 0b01000000;
const GENERATE_NMI           : u8 = 0b10000000;

impl ControlRegister {
	pub fn new() -> ControlRegister {
		ControlRegister {
			value: 0x00
		}
	}

	pub fn contains(&self, flag: u8) -> bool {
		return (self.value & flag) != 0;
	}

	pub fn vram_addr_increment(&self) -> u8 {
		if !self.contains(VRAM_ADD_INCREMENT) {
			return 1;
		}

		32
	}

	pub fn write(&mut self, value: u8) {
		self.value = value;
	}
}

pub struct Ppu {
	palette_table: [u8; 32],
	vram: [u8; 2048],
	oam_data: [u8; 256],
	internal_data_buf: u8,

	pub addr: AddrRegister,
	pub ctrl: ControlRegister,

	mirroring: Mirroring
}

impl Ppu {
	pub fn new(mirroring: Mirroring) -> Ppu {
		Ppu {
			palette_table: [0; 32],
			vram: [0; 2048],
			oam_data: [0; 256],
			internal_data_buf: 0x00,
			addr: AddrRegister::new(),
			ctrl: ControlRegister::new(),
			mirroring
		}
	}

	pub fn increment_vram_addr(&mut self) {
		self.addr.increment(self.ctrl.vram_addr_increment());
	}

	pub fn read(&mut self, rom: &Rom) -> u8 {
		let addr = self.addr.get();
		self.increment_vram_addr();

		match addr {
			0..=0x1FFF => {
				let result = self.internal_data_buf;
				self.internal_data_buf = rom.mapper.read_chr_rom(addr);
				result
			},
           	0x2000..=0x2FFF => {
				let result = self.internal_data_buf;
				self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
				result
			},
           	0x3000..=0x3EFF => panic!("addr space 0x3000..0x3eff is not expected to be used, requested = {} ", addr),
           	0x3F00..=0x3FFF => {
           	    self.palette_table[(addr - 0x3F00) as usize]
           	}
           	_ => panic!("unexpected access to mirrored space {}", addr),
		}
	}

	pub fn write(&mut self, value: u8) {
		let addr = self.addr.get();
		match addr {
			0..=0x1FFF => panic!("Trying to write to chr_rom at {:04x}", addr),
			0x2000..=0x2FFF => {
				self.vram[self.mirror_vram_addr(addr) as usize] = value;
				todo!("Mirror addr");
			},
			0x3000..=0x3EFF => panic!("Addr space 0x3000..0x3EFF is not expected to be used, requested = {:04x} ", addr),
			0x3F00..=0x3FFF => {
				self.palette_table[(addr - 0x3F00) as usize] = value;
			}
			_ => panic!("unexpected access to mirrored space {}", addr),
		}

		self.increment_vram_addr();
	}

	pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
		let mirrored_vram = addr & 0x2FFF; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
       	let vram_index = mirrored_vram - 0x2000; // to vram vector
       	let name_table = vram_index / 0x400; // to the name table index
       	match (&self.mirroring, name_table) {
        	(Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
           	(Mirroring::Horizontal, 2) => vram_index - 0x400,
           	(Mirroring::Horizontal, 1) => vram_index - 0x400,
           	(Mirroring::Horizontal, 3) => vram_index - 0x800,
           	_ => vram_index,
       }
	}
}