use crate::memory::Memory;

pub struct Cpu {
	pc: u16,
	//sp: u8,

	//p: u8,
	a: u8,
	x: u8,
	y: u8,

	n: u8,
	v: u8,
	//b: u8,
	//d: u8,
	//i: u8,
	z: u8,
	c: u8,
}

enum Instruction {
	Adc(u8),
	And(u8),
	AslA,
	Asl(u16),
	Bcc(i8),
	Bcs(i8),
	Beq(i8),
	Bit(u8),
	Bmi(i8),
	Bne(i8),
	Bpl(i8),
	Brk,
	Bvc(i8),
	Bvs(i8),
	Clc,
	Cld,
	Cli,
	Clv,
	Cmp(u8),
	Cpx(u8),
	Cpy(u8),
	Dec(u16),
	Dex,
	Dey,
	Eor(u8),
	Inc(u16),
	Inx,
	Iny,
	Jmp(u16),
	Jsr(u16),
	Lda(u8),
	Ldx(u8),
	Ldy(u8),
	LsrA,
	Lsr(u16),
	Nop,
	Ora(u8),
	Pha,
	Php,
	Pla,
	Plp,
	RolA,
	Rol(u16),
	RorA,
	Ror(u16),
	Rti,
	Rts,
	Sbc(u8),
	Sec,
	Sed,
	Sei,
	Sta(u16),
	Stx(u16),
	Sty(u16),
	Tax,
	Tay,
	Tsx,
	Txa,
	Txs,
	Tya,

}

impl Cpu {
	pub fn new() -> Cpu {
		Cpu {
			pc: 0,
			//sp: 255,

			//p: 0,
			a: 0,
			x: 0,
			y: 0,

			n: 0,
			v: 0,
			//b: 0,
			//d: 0,
			//i: 0,
			z: 0,
			c: 0
		}
	}

	pub fn step(&mut self, memory: &mut Memory) {
		let opcode = self.fetch(memory);

		let instr = self.decode(memory, opcode);

		self.execute(memory, instr);
	}

	fn fetch(&mut self, memory: &Memory) -> u8 {
		let value = memory.read(self.pc);
		self.pc += 1;
		value
	}

	fn fetch_relative(&mut self, memory: &Memory) -> i8 {
		let adress = self.fetch(memory);

		i8::try_from(i16::from(adress) - 128).unwrap() 
	}

	fn fetch_absolute_adress(&mut self, memory: &Memory) -> u16 {
		// Little endian
		u16::from(self.fetch(memory)) + (u16::from(self.fetch(memory)) << 8)
	}

	fn fetch_absolute_indirect_adress(&mut self, memory: &Memory) -> u16 {
		let low_indirect = self.fetch_absolute_adress(memory);

		let high_indirect = (low_indirect & 0xFF00) + ((low_indirect + 1) & 0x00FF); // Do not increment page

		u16::from(memory.read(low_indirect)) + (u16::from(memory.read(high_indirect)) << 8)
	}

	fn fetch_x_indexed_absolute_adress(&mut self, memory: &Memory) -> u16 {
		self.fetch_absolute_adress(memory) + u16::from(self.x)
	}

	fn fetch_y_indexed_absolute_adress(&mut self, memory: &Memory) -> u16 {
		self.fetch_absolute_adress(memory) + u16::from(self.y)
	}

	fn fetch_zero_page_adress(&mut self, memory: &Memory) -> u16 {
		u16::from(self.fetch(memory))
	}

	fn fetch_x_indexed_zero_page_adress(&mut self, memory: &Memory) -> u16 {
		(self.fetch_zero_page_adress(memory) + u16::from(self.x)) & 0x00ff
	}

	fn fetch_y_indexed_zero_page_adress(&mut self, memory: &Memory) -> u16 {
		(self.fetch_zero_page_adress(memory) + u16::from(self.y)) & 0x00ff
	}

	fn fetch_x_indexed_zero_page_indirect_adress(&mut self, memory: &Memory) -> u16 {
		let indirect = self.fetch_x_indexed_zero_page_adress(memory);
		
		debug_assert!(((indirect+1) & 0xff00) == 0); // Next memory loc must be on zero page

		// Little endian
		u16::from(memory.read(indirect)) + (u16::from(memory.read(indirect+1)) << 8)
	}

	fn fetch_zero_page_indirect_y_indexed_adress(&mut self, memory: &Memory) -> u16 {
		let indirect = self.fetch_zero_page_adress(memory);

		// Little endian
		let adress = u16::from(memory.read(indirect)) + (u16::from(memory.read(indirect+1)) << 8);

		adress + u16::from(self.y)
	}

	fn decode(&mut self, memory: &Memory, opcode: u8) -> Instruction {
		match opcode {
			0x69 => Instruction::Adc(self.fetch(memory)),
			0x6D => Instruction::Adc(memory.read(self.fetch_absolute_adress(memory))),
			0x7D => Instruction::Adc(memory.read(self.fetch_x_indexed_absolute_adress(memory))),
			0x79 => Instruction::Adc(memory.read(self.fetch_y_indexed_absolute_adress(memory))),
			0x65 => Instruction::Adc(memory.read(self.fetch_zero_page_adress(memory))),
			0x75 => Instruction::Adc(memory.read(self.fetch_x_indexed_zero_page_adress(memory))),
			0x61 => Instruction::Adc(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))),
			0x71 => Instruction::Adc(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))),
			
			0x29 => Instruction::And(self.fetch(memory)),
			0x2D => Instruction::And(memory.read(self.fetch_absolute_adress(memory))),
			0x3D => Instruction::And(memory.read(self.fetch_x_indexed_absolute_adress(memory))),
			0x39 => Instruction::And(memory.read(self.fetch_y_indexed_absolute_adress(memory))),
			0x25 => Instruction::And(memory.read(self.fetch_zero_page_adress(memory))),
			0x35 => Instruction::And(memory.read(self.fetch_x_indexed_zero_page_adress(memory))),
			0x21 => Instruction::And(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))),
			0x31 => Instruction::And(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))),

			0x0A => Instruction::AslA,
			0x0E => Instruction::Asl(self.fetch_absolute_adress(memory)),
			0x1E => Instruction::Asl(self.fetch_x_indexed_absolute_adress(memory)),
			0x06 => Instruction::Asl(self.fetch_zero_page_adress(memory)),
			0x16 => Instruction::Asl(self.fetch_x_indexed_zero_page_adress(memory)),
			
			0x90 => Instruction::Bcc(self.fetch_relative(memory)),
			0xB0 => Instruction::Bcs(self.fetch_relative(memory)),
			0xF0 => Instruction::Beq(self.fetch_relative(memory)),

			0x24 => Instruction::Bit(memory.read(self.fetch_zero_page_adress(memory))),
			0x2C => Instruction::Bit(memory.read(self.fetch_absolute_adress(memory))),

			0x30 => Instruction::Bmi(self.fetch_relative(memory)),
			0xD0 => Instruction::Bne(self.fetch_relative(memory)),
			0x10 => Instruction::Bpl(self.fetch_relative(memory)),

			0x00 => Instruction::Brk,

			0x50 => Instruction::Bvc(self.fetch_relative(memory)),
			0x70 => Instruction::Bvs(self.fetch_relative(memory)),

			0x18 => Instruction::Clc,
			0xD8 => Instruction::Cld,
			0x58 => Instruction::Cli,
			0xB8 => Instruction::Clv,

			0xC9 => Instruction::Cmp(self.fetch(memory)),
			0xCD => Instruction::Cmp(memory.read(self.fetch_absolute_adress(memory))),
			0xDD => Instruction::Cmp(memory.read(self.fetch_x_indexed_absolute_adress(memory))),
			0xD9 => Instruction::Cmp(memory.read(self.fetch_y_indexed_absolute_adress(memory))),
			0xC5 => Instruction::Cmp(memory.read(self.fetch_zero_page_adress(memory))),
			0xD5 => Instruction::Cmp(memory.read(self.fetch_x_indexed_zero_page_adress(memory))),
			0xC1 => Instruction::Cmp(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))),
			0xD1 => Instruction::Cmp(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))),

			0xE0 => Instruction::Cpx(self.fetch(memory)),
			0xEC => Instruction::Cpx(memory.read(self.fetch_absolute_adress(memory))),
			0xE4 => Instruction::Cpx(memory.read(self.fetch_zero_page_adress(memory))),

			0xC0 => Instruction::Cpy(self.fetch(memory)),
			0xCC => Instruction::Cpy(memory.read(self.fetch_absolute_adress(memory))),
			0xC4 => Instruction::Cpy(memory.read(self.fetch_zero_page_adress(memory))),

			0xCE => Instruction::Dec(self.fetch_absolute_adress(memory)),
			0xDE => Instruction::Dec(self.fetch_x_indexed_absolute_adress(memory)),
			0xC6 => Instruction::Dec(self.fetch_zero_page_adress(memory)),
			0xD6 => Instruction::Dec(self.fetch_x_indexed_zero_page_adress(memory)),

			0xCA => Instruction::Dex,
			0x88 => Instruction::Dey,

			0x49 => Instruction::Eor(self.fetch(memory)),
			0x4D => Instruction::Eor(memory.read(self.fetch_absolute_adress(memory))),
			0x5D => Instruction::Eor(memory.read(self.fetch_x_indexed_absolute_adress(memory))),
			0x59 => Instruction::Eor(memory.read(self.fetch_y_indexed_absolute_adress(memory))),
			0x45 => Instruction::Eor(memory.read(self.fetch_zero_page_adress(memory))),
			0x55 => Instruction::Eor(memory.read(self.fetch_x_indexed_zero_page_adress(memory))),
			0x41 => Instruction::Eor(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))),
			0x51 => Instruction::Eor(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))),

			0xEE => Instruction::Inc(self.fetch_absolute_adress(memory)),
			0xFE => Instruction::Inc(self.fetch_x_indexed_absolute_adress(memory)),
			0xE6 => Instruction::Inc(self.fetch_zero_page_adress(memory)),
			0xF6 => Instruction::Inc(self.fetch_x_indexed_zero_page_adress(memory)),

			0xE8 => Instruction::Inx,
			0xC8 => Instruction::Iny,

			0x4C => Instruction::Jmp(self.fetch_absolute_adress(memory)),
			0x6C => Instruction::Jmp(self.fetch_absolute_indirect_adress(memory)),

			0x20 => Instruction::Jsr(self.fetch_absolute_adress(memory)),

			0xA9 => Instruction::Lda(self.fetch(memory)),
			0xAD => Instruction::Lda(memory.read(self.fetch_absolute_adress(memory))),
			0xBD => Instruction::Lda(memory.read(self.fetch_x_indexed_absolute_adress(memory))),
			0xB9 => Instruction::Lda(memory.read(self.fetch_y_indexed_absolute_adress(memory))),
			0xA5 => Instruction::Lda(memory.read(self.fetch_zero_page_adress(memory))),
			0xB5 => Instruction::Lda(memory.read(self.fetch_x_indexed_zero_page_adress(memory))),
			0xA1 => Instruction::Lda(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))),
			0xB1 => Instruction::Lda(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))),

			0xA2 => Instruction::Ldx(self.fetch(memory)),
			0xAE => Instruction::Ldx(memory.read(self.fetch_absolute_adress(memory))),
			0xBE => Instruction::Ldx(memory.read(self.fetch_y_indexed_absolute_adress(memory))),
			0xA6 => Instruction::Ldx(memory.read(self.fetch_zero_page_adress(memory))),
			0xB6 => Instruction::Ldx(memory.read(self.fetch_y_indexed_zero_page_adress(memory))),

			0xA0 => Instruction::Ldy(self.fetch(memory)),
			0xAC => Instruction::Ldy(memory.read(self.fetch_absolute_adress(memory))),
			0xBC => Instruction::Ldy(memory.read(self.fetch_x_indexed_absolute_adress(memory))),
			0xA4 => Instruction::Ldy(memory.read(self.fetch_zero_page_adress(memory))),
			0xB4 => Instruction::Ldy(memory.read(self.fetch_x_indexed_zero_page_adress(memory))),

			0x4A => Instruction::LsrA,
			0x4E => Instruction::Lsr(self.fetch_absolute_adress(memory)),
			0x5E => Instruction::Lsr(self.fetch_x_indexed_absolute_adress(memory)),
			0x46 => Instruction::Lsr(self.fetch_zero_page_adress(memory)),
			0x56 => Instruction::Lsr(self.fetch_x_indexed_zero_page_adress(memory)),

			0xEA => Instruction::Nop,

			0x09 => Instruction::Ora(self.fetch(memory)),
			0x0D => Instruction::Ora(memory.read(self.fetch_absolute_adress(memory))),
			0x1D => Instruction::Ora(memory.read(self.fetch_x_indexed_absolute_adress(memory))),
			0x19 => Instruction::Ora(memory.read(self.fetch_y_indexed_absolute_adress(memory))),
			0x05 => Instruction::Ora(memory.read(self.fetch_zero_page_adress(memory))),
			0x15 => Instruction::Ora(memory.read(self.fetch_x_indexed_zero_page_adress(memory))),
			0x01 => Instruction::Ora(memory.read(self.fetch_y_indexed_zero_page_adress(memory))),
			0x11 => Instruction::Ora(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))),

			0x48 => Instruction::Pha,
			0x08 => Instruction::Php,
			0x68 => Instruction::Pla,
			0x28 => Instruction::Plp,

			0x2A => Instruction::RolA,
			0x2E => Instruction::Rol(self.fetch_absolute_adress(memory)),
			0x3E => Instruction::Rol(self.fetch_x_indexed_absolute_adress(memory)),
			0x26 => Instruction::Rol(self.fetch_zero_page_adress(memory)),
			0x36 => Instruction::Rol(self.fetch_x_indexed_zero_page_adress(memory)),
			
			0x6A => Instruction::RorA,
			0x6E => Instruction::Ror(self.fetch_absolute_adress(memory)),
			0x7E => Instruction::Ror(self.fetch_x_indexed_absolute_adress(memory)),
			0x66 => Instruction::Ror(self.fetch_zero_page_adress(memory)),
			0x76 => Instruction::Ror(self.fetch_x_indexed_zero_page_adress(memory)),

			0x40 => Instruction::Rti,
			0x60 => Instruction::Rts,

			0xE9 => Instruction::Sbc(self.fetch(memory)),
			0xED => Instruction::Sbc(memory.read(self.fetch_absolute_adress(memory))),
			0xFD => Instruction::Sbc(memory.read(self.fetch_x_indexed_absolute_adress(memory))),
			0xF9 => Instruction::Sbc(memory.read(self.fetch_y_indexed_absolute_adress(memory))),
			0xE5 => Instruction::Sbc(memory.read(self.fetch_zero_page_adress(memory))),
			0xF5 => Instruction::Sbc(memory.read(self.fetch_x_indexed_zero_page_adress(memory))),
			0xE1 => Instruction::Sbc(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))),
			0xF1 => Instruction::Sbc(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))),

			0x38 => Instruction::Sec,
			0xF8 => Instruction::Sed,
			0x78 => Instruction::Sei,

			0x8D => Instruction::Sta(self.fetch_absolute_adress(memory)),
			0x9D => Instruction::Sta(self.fetch_x_indexed_absolute_adress(memory)),
			0x99 => Instruction::Sta(self.fetch_y_indexed_absolute_adress(memory)),
			0x85 => Instruction::Sta(self.fetch_zero_page_adress(memory)),
			0x95 => Instruction::Sta(self.fetch_x_indexed_zero_page_adress(memory)),
			0x81 => Instruction::Sta(self.fetch_x_indexed_zero_page_indirect_adress(memory)),
			0x91 => Instruction::Sta(self.fetch_zero_page_indirect_y_indexed_adress(memory)),

			0x8E => Instruction::Stx(self.fetch_absolute_adress(memory)),
			0x86 => Instruction::Stx(self.fetch_zero_page_adress(memory)),
			0x96 => Instruction::Stx(self.fetch_y_indexed_zero_page_adress(memory)),

			0x8C => Instruction::Sty(self.fetch_absolute_adress(memory)),
			0x84 => Instruction::Sty(self.fetch_zero_page_adress(memory)),
			0x94 => Instruction::Sty(self.fetch_x_indexed_zero_page_adress(memory)),

			0xAA => Instruction::Tax,
			0xA8 => Instruction::Tay,
			0xBA => Instruction::Tsx,
			0x8A => Instruction::Txa,
			0x9A => Instruction::Txs,
			0x98 => Instruction::Tya,

			_ => {
				panic!("Opcode '{}' not implemented", opcode)
			}
		}
	}

	fn execute(&mut self, memory: &mut Memory, instruction: Instruction) {
		match instruction {
			Instruction::Adc(value) => {
				self.a = self.apply_adc_op(value);
			},
			Instruction::And(value) => {
				self.a = self.apply_and_op(value);
			}
			Instruction::AslA => {
				self.a = self.apply_asl_op(self.a);
			},
			Instruction::Asl(adress) => {
				let value = memory.read(adress);
				
				memory.write(adress, self.apply_asl_op(value));
			},
			Instruction::Bcc(offset) => {
				self.pc = self.apply_bcc_op(self.pc, offset);
			},
			Instruction::Bcs(offset) => {
				self.pc = self.apply_bcs_op(self.pc, offset);
			},
			Instruction::Beq(offset) => {
				self.pc = self.apply_beq_op(self.pc, offset);
			},
			Instruction::Bit(value) => {
				self.apply_bit_op(value);
			},
			Instruction::Bmi(offset) => {
				self.pc = self.apply_bmi_op(self.pc, offset);
			},
			Instruction::Bne(offset) => {
				self.pc = self.apply_bne_op(self.pc, offset);
			},
			Instruction::Bpl(offset) => {
				self.pc = self.apply_bpl_op(self.pc, offset);
			},
			Instruction::Brk => todo!("TODO: Brk"),
			Instruction::Bvc(_) => todo!("TODO: Bvc"),
			Instruction::Bvs(_) => todo!("TODO: Bvs"),
			Instruction::Clc => todo!("TODO: Clc"),
			Instruction::Cld => todo!("TODO: Cld"),
			Instruction::Cli => todo!("TODO: Cli"),
			Instruction::Clv => todo!("TODO: Clv"),
			Instruction::Cmp(_) => todo!("TODO: Cmp"),
			Instruction::Cpx(_) => todo!("TODO: Cpx"),
			Instruction::Cpy(_) => todo!("TODO: Cpy"),
			Instruction::Dec(_) => todo!("TODO: Dec"),
			Instruction::Dex => todo!("TODO: Dex"),
			Instruction::Dey => todo!("TODO: Dey"),
			Instruction::Eor(_) => todo!("TODO: Eor"),
			Instruction::Inc(_) => todo!("TODO: Inc"),
			Instruction::Inx => todo!("TODO: Inx"),
			Instruction::Iny => todo!("TODO: Iny"),
			Instruction::Jmp(_) => todo!("TODO: Jmp"),
			Instruction::Jsr(_) => todo!("TODO: Jsr"),
			Instruction::Lda(_) => todo!("TODO: Lda"),
			Instruction::Ldx(_) => todo!("TODO: Ldx"),
			Instruction::Ldy(_) => todo!("TODO: Ldy"),
			Instruction::LsrA => todo!("Todo: LsrA"),
			Instruction::Lsr(_) => todo!("TODO: Lsr"),
			Instruction::Ora(_) => todo!("TODO: Ora"),
			Instruction::Pha => todo!("TODO: Pha"),
			Instruction::Php => todo!("TODO: Php"),
			Instruction::Pla => todo!("TODO: Pla"),
			Instruction::Plp => todo!("TODO: Plp"),
			Instruction::RolA => todo!("TODO: RolA"),
			Instruction::Rol(_) => todo!("TODO: Rol"),
			Instruction::RorA => todo!("TODO: Ror"),
			Instruction::Ror(_) => todo!("TODO: Ror"),
			Instruction::Rti => todo!("TODO: Rti"),
			Instruction::Rts => todo!("TODO: Rts"),
			Instruction::Sbc(_) => todo!("TODO: Sbc"),
			Instruction::Sec => todo!("TODO: Sec"),
			Instruction::Sed => todo!("TODO: Sed"),
			Instruction::Sei => todo!("TODO: Sei"),
			Instruction::Sta(_) => todo!("TODO: Sta"),
			Instruction::Stx(_) => todo!("TODO: Stx"),
			Instruction::Sty(_) => todo!("TODO: Sty"),
			Instruction::Tax => todo!("TODO: Tax"),
			Instruction::Tay => todo!("TODO: Tay"),
			Instruction::Tsx => todo!("TODO: Tsx"),
			Instruction::Txa => todo!("TODO: Txa"),
			Instruction::Txs => todo!("TODO: Txs"),
			Instruction::Tya => todo!("TODO: Tya"),

			Instruction::Nop => {}
		}
	}

	fn apply_adc_op(&mut self, value: u8) -> u8 {
		let (temp, overflowed_1) = u8::overflowing_add(value, value);
		let (result, overflowed_2) = u8::overflowing_add(temp, self.c);
		
		self.c = u8::from(overflowed_1 || overflowed_2);
		self.v = u8::from((value & 0x80) != (result & 0x80));
		self.n = u8::from(result & 0x80 == 0x80);
		self.z = u8::from(result == 0);
		
		result
	}

	fn apply_and_op(&mut self, value: u8) -> u8 {
		let result = self.a & value;

		self.z = u8::from(result == 0);
		self.n = u8::from(result & 0x80 == 0x80);

		result
	}

	fn apply_asl_op(&mut self, value: u8) -> u8 {
		self.c = (value & 0x80) >> 7;

		let result = (value & 0x7F) << 1;

		self.n = (result & 0x80) >> 7;
		self.z = u8::from(result == 0);

		result
	}

	fn apply_bcc_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.c == 0 {
			return u16::try_from(i32::from(pc) + i32::from(offset)).unwrap();
		}
		
		pc
	}

	fn apply_bcs_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.c != 0 {
			return u16::try_from(i32::from(pc) + i32::from(offset)).unwrap();
		}
		
		pc
	}

	fn apply_beq_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.z != 0 {
			return u16::try_from(i32::from(pc) + i32::from(offset)).unwrap();
		}
		
		pc
	}

	fn apply_bit_op(&mut self, value: u8) {
		self.n = (value & 0x80) >> 7;
		self.v = (value & 0x40) >> 6;

		self.z = u8::from((self.a & value) == 0);
	}

	fn apply_bmi_op(&mut self, pc: u16, offset: i8) -> u16 {	
		if self.n != 0 {
			return u16::try_from(i32::from(pc) + i32::from(offset)).unwrap();
		}
		
		pc
	}

	fn apply_bne_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.z == 0 {
			return u16::try_from(i32::from(pc) + i32::from(offset)).unwrap();
		}
		
		pc
	}

	fn apply_bpl_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.n == 0 {
			return u16::try_from(i32::from(pc) + i32::from(offset)).unwrap();
		}
		
		pc
	}
}