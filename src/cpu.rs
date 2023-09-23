use crate::memory::Memory;

pub struct Cpu {
	pc: u16,
	sp: u8,

	a: u8,
	x: u8,
	y: u8,

	n: u8,
	v: u8,
	b: u8,
	d: u8,
	i: u8,
	z: u8,
	c: u8,

	extra_cycle: u8
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
			pc: 0xFFFC,
			sp: 0xFF,

			a: 0,
			x: 0,
			y: 0,

			n: 0,
			v: 0,
			b: 0,
			d: 0,
			i: 0,
			z: 0,
			c: 0,

			extra_cycle: 0,
		}
	}

	pub fn step(&mut self, memory: &mut Memory) -> u8 {
		let opcode = self.fetch(memory);

		let (instr, cycle) = self.decode(memory, opcode);

		self.extra_cycle = 0;
		self.execute(memory, instr);

		cycle + self.extra_cycle
	}

	fn stack_push(&mut self, memory: &mut Memory, value: u8) {
		memory.write(0x0100 + u16::from(self.sp), value);

		self.sp -= 1;
	}

	fn stack_pop(&mut self, memory: &Memory) -> u8 {
		self.sp += 1;
		
		memory.read(0x0100 + u16::from(self.sp))
	}

	fn set_status(&mut self, p: u8) {
		self.n = p >> 7;
		self.v = (p & 0x40) >> 6;
		self.b = (p & 0x10) >> 4;
		self.d = (p & 0x08) >> 3;
		self.i = (p & 0x04) >> 2;
		self.z = (p & 0x02) >> 1;
		self.c = p & 0x01;
	}

	fn get_status(&mut self) -> u8 {
		(self.n << 7) + (self.v << 6) + (self.b << 4) + (self.d << 3) + (self.i << 2) + (self.z << 1) + self.c
	}

	fn cross(origin: u16, next: u16) -> bool {
		(origin & 0xFF00) != (next & 0xFF00)
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
		let absolute = self.fetch_absolute_adress(memory);
		let adress = absolute + u16::from(self.x);

		self.extra_cycle = u8::from(Cpu::cross(absolute, adress));

		adress
	}

	fn fetch_y_indexed_absolute_adress(&mut self, memory: &Memory) -> u16 {
		let absolute = self.fetch_absolute_adress(memory) + u16::from(self.y);
		let adress = absolute + u16::from(self.y);

		self.extra_cycle = u8::from(Cpu::cross(absolute, adress));

		adress
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
		let pointer = self.fetch_zero_page_adress(memory);

		// Little endian
		let indirect = u16::from(memory.read(pointer)) + (u16::from(memory.read(pointer+1)) << 8);
		let adress = indirect + u16::from(self.y);

		self.extra_cycle = u8::from(Cpu::cross(indirect, adress)); // Cross

		adress
	}

	fn decode(&mut self, memory: &Memory, opcode: u8) -> (Instruction, u8) {
		match opcode {
			0x69 => (Instruction::Adc(self.fetch(memory)), 2),
			0x6D => (Instruction::Adc(memory.read(self.fetch_absolute_adress(memory))), 4),
			0x7D => (Instruction::Adc(memory.read(self.fetch_x_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0x79 => (Instruction::Adc(memory.read(self.fetch_y_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0x65 => (Instruction::Adc(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0x75 => (Instruction::Adc(memory.read(self.fetch_x_indexed_zero_page_adress(memory))), 4),
			0x61 => (Instruction::Adc(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))), 6),
			0x71 => (Instruction::Adc(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))), 5 + self.extra_cycle),
			
			0x29 => (Instruction::And(self.fetch(memory)), 2),
			0x2D => (Instruction::And(memory.read(self.fetch_absolute_adress(memory))), 4),
			0x3D => (Instruction::And(memory.read(self.fetch_x_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0x39 => (Instruction::And(memory.read(self.fetch_y_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0x25 => (Instruction::And(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0x35 => (Instruction::And(memory.read(self.fetch_x_indexed_zero_page_adress(memory))), 4),
			0x21 => (Instruction::And(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))), 6),
			0x31 => (Instruction::And(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))), 5 + self.extra_cycle),

			0x0A => (Instruction::AslA, 2),
			0x0E => (Instruction::Asl(self.fetch_absolute_adress(memory)), 6),
			0x1E => (Instruction::Asl(self.fetch_x_indexed_absolute_adress(memory)), 7),
			0x06 => (Instruction::Asl(self.fetch_zero_page_adress(memory)), 5),
			0x16 => (Instruction::Asl(self.fetch_x_indexed_zero_page_adress(memory)), 6),

			0x90 => (Instruction::Bcc(self.fetch_relative(memory)), 2 /* + p + t */),
			0xB0 => (Instruction::Bcs(self.fetch_relative(memory)), 2 /* + p + t */),
			0xF0 => (Instruction::Beq(self.fetch_relative(memory)), 2 /* + p + t */),

			0x24 => (Instruction::Bit(memory.read(self.fetch_zero_page_adress(memory))), 4),
			0x2C => (Instruction::Bit(memory.read(self.fetch_absolute_adress(memory))), 3),

			0x30 => (Instruction::Bmi(self.fetch_relative(memory)), 2 /* + p + t */),
			0xD0 => (Instruction::Bne(self.fetch_relative(memory)), 2 /* + p + t */),
			0x10 => (Instruction::Bpl(self.fetch_relative(memory)), 2 /* + p + t */),

			0x00 => (Instruction::Brk, 7),

			0x50 => (Instruction::Bvc(self.fetch_relative(memory)), 2 /* + p + t */),
			0x70 => (Instruction::Bvs(self.fetch_relative(memory)), 2 /* + p + t */),

			0x18 => (Instruction::Clc, 2),
			0xD8 => (Instruction::Cld, 2),
			0x58 => (Instruction::Cli, 2),
			0xB8 => (Instruction::Clv, 2),

			0xC9 => (Instruction::Cmp(self.fetch(memory)), 2),
			0xCD => (Instruction::Cmp(memory.read(self.fetch_absolute_adress(memory))), 4),
			0xDD => (Instruction::Cmp(memory.read(self.fetch_x_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0xD9 => (Instruction::Cmp(memory.read(self.fetch_y_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0xC5 => (Instruction::Cmp(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0xD5 => (Instruction::Cmp(memory.read(self.fetch_x_indexed_zero_page_adress(memory))), 4),
			0xC1 => (Instruction::Cmp(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))), 6),
			0xD1 => (Instruction::Cmp(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))), 5 + self.extra_cycle),

			0xE0 => (Instruction::Cpx(self.fetch(memory)), 2),
			0xEC => (Instruction::Cpx(memory.read(self.fetch_absolute_adress(memory))), 4),
			0xE4 => (Instruction::Cpx(memory.read(self.fetch_zero_page_adress(memory))), 3),

			0xC0 => (Instruction::Cpy(self.fetch(memory)), 2),
			0xCC => (Instruction::Cpy(memory.read(self.fetch_absolute_adress(memory))), 4),
			0xC4 => (Instruction::Cpy(memory.read(self.fetch_zero_page_adress(memory))), 3),

			0xCE => (Instruction::Dec(self.fetch_absolute_adress(memory)), 6),
			0xDE => (Instruction::Dec(self.fetch_x_indexed_absolute_adress(memory)), 7),
			0xC6 => (Instruction::Dec(self.fetch_zero_page_adress(memory)), 5),
			0xD6 => (Instruction::Dec(self.fetch_x_indexed_zero_page_adress(memory)), 6),

			0xCA => (Instruction::Dex, 2),
			0x88 => (Instruction::Dey, 2),

			0x49 => (Instruction::Eor(self.fetch(memory)), 2),
			0x4D => (Instruction::Eor(memory.read(self.fetch_absolute_adress(memory))), 4),
			0x5D => (Instruction::Eor(memory.read(self.fetch_x_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0x59 => (Instruction::Eor(memory.read(self.fetch_y_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0x45 => (Instruction::Eor(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0x55 => (Instruction::Eor(memory.read(self.fetch_x_indexed_zero_page_adress(memory))), 4),
			0x41 => (Instruction::Eor(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))), 6),
			0x51 => (Instruction::Eor(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))), 5 + self.extra_cycle),

			0xEE => (Instruction::Inc(self.fetch_absolute_adress(memory)), 6),
			0xFE => (Instruction::Inc(self.fetch_x_indexed_absolute_adress(memory)), 7),
			0xE6 => (Instruction::Inc(self.fetch_zero_page_adress(memory)), 5),
			0xF6 => (Instruction::Inc(self.fetch_x_indexed_zero_page_adress(memory)), 6),

			0xE8 => (Instruction::Inx, 2),
			0xC8 => (Instruction::Iny, 2),

			0x4C => (Instruction::Jmp(self.fetch_absolute_adress(memory)), 3),
			0x6C => (Instruction::Jmp(self.fetch_absolute_indirect_adress(memory)), 5),

			0x20 => (Instruction::Jsr(self.fetch_absolute_adress(memory)), 6),

			0xA9 => (Instruction::Lda(self.fetch(memory)), 2),
			0xAD => (Instruction::Lda(memory.read(self.fetch_absolute_adress(memory))), 4),
			0xBD => (Instruction::Lda(memory.read(self.fetch_x_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0xB9 => (Instruction::Lda(memory.read(self.fetch_y_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0xA5 => (Instruction::Lda(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0xB5 => (Instruction::Lda(memory.read(self.fetch_x_indexed_zero_page_adress(memory))), 4),
			0xA1 => (Instruction::Lda(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))), 6),
			0xB1 => (Instruction::Lda(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))), 5 + self.extra_cycle),

			0xA2 => (Instruction::Ldx(self.fetch(memory)), 2),
			0xAE => (Instruction::Ldx(memory.read(self.fetch_absolute_adress(memory))), 4),
			0xBE => (Instruction::Ldx(memory.read(self.fetch_y_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0xA6 => (Instruction::Ldx(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0xB6 => (Instruction::Ldx(memory.read(self.fetch_y_indexed_zero_page_adress(memory))), 4),

			0xA0 => (Instruction::Ldy(self.fetch(memory)), 2),
			0xAC => (Instruction::Ldy(memory.read(self.fetch_absolute_adress(memory))), 4),
			0xBC => (Instruction::Ldy(memory.read(self.fetch_x_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0xA4 => (Instruction::Ldy(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0xB4 => (Instruction::Ldy(memory.read(self.fetch_x_indexed_zero_page_adress(memory))), 4),

			0x4A => (Instruction::LsrA, 2),
			0x4E => (Instruction::Lsr(self.fetch_absolute_adress(memory)), 6),
			0x5E => (Instruction::Lsr(self.fetch_x_indexed_absolute_adress(memory)), 7),
			0x46 => (Instruction::Lsr(self.fetch_zero_page_adress(memory)), 5),
			0x56 => (Instruction::Lsr(self.fetch_x_indexed_zero_page_adress(memory)), 6),

			0xEA => (Instruction::Nop, 2),

			0x09 => (Instruction::Ora(self.fetch(memory)), 2),
			0x0D => (Instruction::Ora(memory.read(self.fetch_absolute_adress(memory))), 4),
			0x1D => (Instruction::Ora(memory.read(self.fetch_x_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0x19 => (Instruction::Ora(memory.read(self.fetch_y_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0x05 => (Instruction::Ora(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0x15 => (Instruction::Ora(memory.read(self.fetch_x_indexed_zero_page_adress(memory))), 4),
			0x01 => (Instruction::Ora(memory.read(self.fetch_y_indexed_zero_page_adress(memory))), 6),
			0x11 => (Instruction::Ora(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))), 5 + self.extra_cycle),

			0x48 => (Instruction::Pha, 3),
			0x08 => (Instruction::Php, 3),
			0x68 => (Instruction::Pla, 4),
			0x28 => (Instruction::Plp, 4),

			0x2A => (Instruction::RolA, 2),
			0x2E => (Instruction::Rol(self.fetch_absolute_adress(memory)), 6),
			0x3E => (Instruction::Rol(self.fetch_x_indexed_absolute_adress(memory)), 7),
			0x26 => (Instruction::Rol(self.fetch_zero_page_adress(memory)), 5),
			0x36 => (Instruction::Rol(self.fetch_x_indexed_zero_page_adress(memory)), 6),
			
			0x6A => (Instruction::RorA, 2),
			0x6E => (Instruction::Ror(self.fetch_absolute_adress(memory)), 6),
			0x7E => (Instruction::Ror(self.fetch_x_indexed_absolute_adress(memory)), 7),
			0x66 => (Instruction::Ror(self.fetch_zero_page_adress(memory)), 5),
			0x76 => (Instruction::Ror(self.fetch_x_indexed_zero_page_adress(memory)), 6),

			0x40 => (Instruction::Rti, 6),
			0x60 => (Instruction::Rts, 6),

			0xE9 => (Instruction::Sbc(self.fetch(memory)), 2),
			0xED => (Instruction::Sbc(memory.read(self.fetch_absolute_adress(memory))), 4),
			0xFD => (Instruction::Sbc(memory.read(self.fetch_x_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0xF9 => (Instruction::Sbc(memory.read(self.fetch_y_indexed_absolute_adress(memory))), 4 + self.extra_cycle),
			0xE5 => (Instruction::Sbc(memory.read(self.fetch_zero_page_adress(memory))), 3),
			0xF5 => (Instruction::Sbc(memory.read(self.fetch_x_indexed_zero_page_adress(memory))), 4),
			0xE1 => (Instruction::Sbc(memory.read(self.fetch_x_indexed_zero_page_indirect_adress(memory))), 6),
			0xF1 => (Instruction::Sbc(memory.read(self.fetch_zero_page_indirect_y_indexed_adress(memory))), 5 + self.extra_cycle),

			0x38 => (Instruction::Sec, 2),
			0xF8 => (Instruction::Sed, 2),
			0x78 => (Instruction::Sei, 2),

			0x8D => (Instruction::Sta(self.fetch_absolute_adress(memory)), 4),
			0x9D => (Instruction::Sta(self.fetch_x_indexed_absolute_adress(memory)), 5),
			0x99 => (Instruction::Sta(self.fetch_y_indexed_absolute_adress(memory)), 5),
			0x85 => (Instruction::Sta(self.fetch_zero_page_adress(memory)), 3),
			0x95 => (Instruction::Sta(self.fetch_x_indexed_zero_page_adress(memory)), 4),
			0x81 => (Instruction::Sta(self.fetch_x_indexed_zero_page_indirect_adress(memory)), 6),
			0x91 => (Instruction::Sta(self.fetch_zero_page_indirect_y_indexed_adress(memory)), 6),

			0x8E => (Instruction::Stx(self.fetch_absolute_adress(memory)), 4),
			0x86 => (Instruction::Stx(self.fetch_zero_page_adress(memory)), 3),
			0x96 => (Instruction::Stx(self.fetch_y_indexed_zero_page_adress(memory)), 4),

			0x8C => (Instruction::Sty(self.fetch_absolute_adress(memory)), 4),
			0x84 => (Instruction::Sty(self.fetch_zero_page_adress(memory)), 3),
			0x94 => (Instruction::Sty(self.fetch_x_indexed_zero_page_adress(memory)), 4),

			0xAA => (Instruction::Tax, 2),
			0xA8 => (Instruction::Tay, 2),
			0xBA => (Instruction::Tsx, 2),
			0x8A => (Instruction::Txa, 2),
			0x9A => (Instruction::Txs, 2),
			0x98 => (Instruction::Tya, 2),

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
			Instruction::Bit(value) => self.apply_bit_op(value),
			Instruction::Bmi(offset) => {
				self.pc = self.apply_bmi_op(self.pc, offset);
			},
			Instruction::Bne(offset) => {
				self.pc = self.apply_bne_op(self.pc, offset);
			},
			Instruction::Bpl(offset) => {
				self.pc = self.apply_bpl_op(self.pc, offset);
			},
			Instruction::Brk => self.apply_brk_op(memory),
			Instruction::Bvc(offset) => {
				self.pc = self.apply_bvc_op(self.pc, offset)
			},
			Instruction::Bvs(offset) => {
				self.pc = self.apply_bvs_op(self.pc, offset);
			},
			Instruction::Clc => self.c = 0,
			Instruction::Cld => self.d = 0,
			Instruction::Cli => self.i = 0,
			Instruction::Clv => self.v = 0,
			Instruction::Cmp(value) => self.apply_cmp_op(self.a, value),
			Instruction::Cpx(value) => self.apply_cmp_op(self.x, value),
			Instruction::Cpy(value) => self.apply_cmp_op(self.y, value),
			Instruction::Dec(adress) => {
				let value = memory.read(adress);

				memory.write(adress, self.apply_dec_op(value));
			},
			Instruction::Dex => self.x = self.apply_dec_op(self.x),
			Instruction::Dey => self.x = self.apply_dec_op(self.y),
			Instruction::Eor(value) => {
				self.a = self.apply_eor_op(value);
			},
			Instruction::Inc(adress) => {
				let value = memory.read(adress);

				memory.write(adress, self.apply_inc_op(value));
			},
			Instruction::Inx => {
				self.x = self.apply_inc_op(self.x);
			},
			Instruction::Iny => {
				self.y = self.apply_inc_op(self.y);
			},
			Instruction::Jmp(adress) => self.pc = adress,
			Instruction::Jsr(adress) => {
				self.apply_jsr_op(memory);

				self.pc = adress;
			},
			Instruction::Lda(value) => {
				self.a = self.apply_ld_op(value);
			},
			Instruction::Ldx(value) => {
				self.x = self.apply_ld_op(value);
			},
			Instruction::Ldy(value) => {
				self.y = self.apply_ld_op(value);
			},
			Instruction::LsrA => {
				self.a = self.apply_lsr_op(self.a);
			},
			Instruction::Lsr(adress) => {
				let value = memory.read(adress);

				memory.write(adress, self.apply_lsr_op(value));
			},
			Instruction::Ora(value) => {
				self.a = self.apply_ora_op(value);
			},
			Instruction::Pha => self.apply_pha_op(memory),
			Instruction::Php => self.apply_php_op(memory),
			Instruction::Pla => self.apply_pla_op(memory),
			Instruction::Plp => self.apply_plp_op(memory),
			Instruction::RolA => {
				self.a = self.apply_rol_op(self.a);
			}
			Instruction::Rol(adress) => {
				let value = memory.read(adress);

				memory.write(adress, self.apply_rol_op(value));
			},
			Instruction::RorA => {
				self.a = self.apply_ror_op(self.a);
			},
			Instruction::Ror(adress) => {
				let value = memory.read(adress);

				memory.write(adress, self.apply_ror_op(value));
			},
			Instruction::Rti => self.apply_rti_op(memory),
			Instruction::Rts => {
				self.pc = self.apply_rts_op(memory);
			},
			Instruction::Sbc(value) => {
				self.a = self.apply_sbc_op(value);
			},
			Instruction::Sec => self.c = 1,
			Instruction::Sed => self.d = 1,
			Instruction::Sei => self.i = 1,
			Instruction::Sta(adress) => {
				memory.write(adress, self.a);
			},
			Instruction::Stx(adress) => {
				memory.write(adress, self.x)
			},
			Instruction::Sty(adress) => {
				memory.write(adress, self.y);
			},
			Instruction::Tax => {
				self.x = self.a;
				self.z = u8::from(self.x == 0);
				self.n = self.x >> 7;
			},
			Instruction::Tay => {
				self.y = self.a;
				self.z = u8::from(self.y == 0);
				self.n = self.y >> 7;
			},
			Instruction::Tsx => {
				self.x = self.sp;
				self.z = u8::from(self.x == 0);
				self.n = self.x >> 7;
			},
			Instruction::Txa => {
				self.a = self.x;
				self.z = u8::from(self.a == 0);
				self.n = self.a >> 7;
			},
			Instruction::Txs => {
				self.sp = self.x;
				self.z = u8::from(self.x == 0);
				self.n = self.x >> 7;
			},
			Instruction::Tya => {
				self.a = self.y;
				self.z = u8::from(self.y == 0);
				self.n = self.y >> 7;
			},

			Instruction::Nop => {}
		}
	}

	fn apply_branch(&mut self, pc: u16, offset: i8) -> u16 {
		let adress = u16::try_from(i32::from(pc) + i32::from(offset)).unwrap();

		self.extra_cycle = 1 + u8::from(Cpu::cross(pc, adress));

		adress
	}

	fn apply_adc_op(&mut self, value: u8) -> u8 {
		let (temp, overflowed_1) = u8::overflowing_add(self.a, value);
		let (result, overflowed_2) = u8::overflowing_add(temp, self.c);
		
		self.c = u8::from(overflowed_1 || overflowed_2);
		self.v = u8::from((value & 0x80) != (result & 0x80));
		self.n = result >> 7;
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

		self.n = result >> 7;
		self.z = u8::from(result == 0);

		result
	}

	fn apply_bcc_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.c == 0 {
			return self.apply_branch(pc, offset);
		}
		
		pc
	}

	fn apply_bcs_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.c != 0 {
			return self.apply_branch(pc, offset);
		}
		
		pc
	}

	fn apply_beq_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.z != 0 {
			return self.apply_branch(pc, offset);
		}
		
		pc
	}

	fn apply_bit_op(&mut self, value: u8) {
		self.n = value >> 7;
		self.v = (value & 0x40) >> 6;

		self.z = u8::from((self.a & value) == 0);
	}

	fn apply_bmi_op(&mut self, pc: u16, offset: i8) -> u16 {	
		if self.n != 0 {
			return self.apply_branch(pc, offset);
		}
		
		pc
	}

	fn apply_bne_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.z == 0 {
			return self.apply_branch(pc, offset);
		}
		
		pc
	}

	fn apply_bpl_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.n == 0 {
			return self.apply_branch(pc, offset);
		}
		
		pc
	}

	fn apply_brk_op(&mut self, memory: &mut Memory) {
		self.apply_jsr_op(memory);
		let p = self.get_status();
		self.stack_push(memory, p);

		self.pc = 0xFFFE;
	}

	fn apply_bvc_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.v == 0 {
			return self.apply_branch(pc, offset);
		}
		
		pc
	}

	fn apply_bvs_op(&mut self, pc: u16, offset: i8) -> u16 {
		if self.v != 0 {
			return self.apply_branch(pc, offset);
		}
		
		pc
	}

	fn apply_cmp_op(&mut self, register: u8, value: u8) {
		let (result, underflow) = register.overflowing_sub(value);
		self.z = u8::from(result == 0);
		self.n = result >> 7;
		self.c = u8::from(!underflow);
	}

	fn apply_dec_op(&mut self, value: u8) -> u8 {
		let (result, _) = value.overflowing_sub(1);

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		result
	}

	fn apply_eor_op(&mut self, value: u8) -> u8 {
		let result = self.a ^ value;

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		result
	}

	fn apply_inc_op(&mut self, value: u8) -> u8 {
		let (result, _) = value.overflowing_add(1);

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		result
	}

	fn apply_jsr_op(&mut self, memory: &mut Memory) {
		let low_pc = u8::try_from(self.pc & 0x00FF).unwrap();
		let high_pc = u8::try_from((self.pc & 0xFF00) >> 8).unwrap();

		self.stack_push(memory, high_pc);
		self.stack_push(memory, low_pc);
	}

	fn apply_ld_op(&mut self, value: u8) -> u8 {
		self.z = u8::from(value == 0);
		self.n = value >> 7;

		value
	}

	fn apply_lsr_op(&mut self, value: u8) -> u8 {
		self.c = value & 0x01;
		self.n = 0;

		let result = value >> 1;
		self.z = u8::from(result == 0);

		result
	}

	fn apply_ora_op(&mut self, value: u8) -> u8 {
		let result = value | self.a;

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		result
	}

	fn apply_pha_op(&mut self, memory: &mut Memory) {
		self.stack_push(memory, self.a);
	}

	fn apply_php_op(&mut self, memory: &mut Memory) {
		let p = self.get_status();
		
		self.stack_push(memory, p);
	}

	fn apply_pla_op(&mut self, memory: &Memory) {
		self.a = self.stack_pop(memory);
	}

	fn apply_plp_op(&mut self, memory: &Memory) {
		let p = self.stack_pop(memory);

		self.set_status(p);
	}

	fn apply_rol_op(&mut self, value: u8) -> u8 {
		let result = (value << 1) + self.c;
		self.c = value >> 7;
		self.n = (value & 0x40) >> 6;
		self.z = u8::from(result == 0);

		result
	}

	fn apply_ror_op(&mut self, value: u8) -> u8 {
		let result = (self.c << 7) + (value >> 1);
		self.n = self.c;
		self.c = value & 0x01;
		self.z = u8::from(result == 0);

		result
	}

	fn apply_rti_op(&mut self, memory: &Memory) {
		let p = self.stack_pop(memory);
		self.pc = self.apply_rts_op(memory);

		self.set_status(p);
	}

	fn apply_rts_op(&mut self, memory: &Memory) -> u16 {
		let low_pc = u16::from(self.stack_pop(memory));
		let high_pc = u16::from(self.stack_pop(memory));

		(high_pc << 8) + low_pc
	}

	fn apply_sbc_op(&mut self, value: u8) -> u8 {
		let (temp, overflowed_1) = u8::overflowing_sub(self.a, value);
		let (result, overflowed_2) = u8::overflowing_add(temp, 1 - self.c);
		
		self.c = !result >> 7; // Greater or equal to 0
		self.v = u8::from(overflowed_1 || overflowed_2);
		self.n = result >> 7;
		self.z = u8::from(result == 0);
		
		result
	}
}


#[cfg(test)]
mod tests {
	use crate::mapper::Mapper;

	use super::*;

	#[test]
	fn cross() {
		assert_eq!(Cpu::cross(0xABCD, 0xABCE), false);
		assert_eq!(Cpu::cross(0x00FF, 0x0100), true);
		assert_eq!(Cpu::cross(0xAB00, 0xFF00), true);
	}

	#[test]
	fn absolute_adress_mode() {
		let mut cpu = Cpu::new();
		cpu.pc = 0x0000;

		let mut memory = Memory::new(<dyn Mapper>::from_id(0x0, vec![], vec![]));
		memory.write(0x0000, 0xCD);
		memory.write(0x0001, 0xAB);

		assert_eq!(cpu.fetch_absolute_adress(&memory), 0x0ABCD);
	}

	#[test]
	fn x_indexed_absolute_adress_mode() {
		let mut cpu = Cpu::new();
		cpu.pc = 0x0000;
		cpu.x = 0x01;

		let mut memory = Memory::new(<dyn Mapper>::from_id(0x0, vec![], vec![]));
		memory.write(0x0000, 0xFF);
		memory.write(0x0001, 0x00);

		assert_eq!(cpu.fetch_x_indexed_absolute_adress(&memory), 0x0100);
		assert_eq!(cpu.extra_cycle, 1);
	}

	#[test]
	fn adc_op() {
		// TODO: need more testing on flags
		let mut cpu = Cpu::new();
		
		cpu.a = 0x01;
		assert_eq!(cpu.apply_adc_op(0xFE), 0xFF);
		assert_eq!(cpu.c, 0);
	
		cpu.a = 0x03;
		assert_eq!(cpu.apply_adc_op(0xFE), 0x01);
		assert_eq!(cpu.c, 1);
	}

	#[test]
	fn cmp_op() {
		let mut cpu = Cpu::new();
		cpu.a = 0x10; // Set accumulator

		cpu.apply_cmp_op(cpu.a, 0x10);
		assert_eq!(cpu.z, 1);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);

		cpu.apply_cmp_op(cpu.a, 0x09);
		assert_eq!(cpu.z, 0);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);

		cpu.apply_cmp_op(cpu.a, 0x11);
		assert_eq!(cpu.z, 0);
		assert_eq!(cpu.c, 0);
		assert_eq!(cpu.n, 1);

		assert_eq!(cpu.a, 0x10);
	}

	#[test]
	fn lsr_op() {
		let mut cpu = Cpu::new();
		cpu.n = 1;

		assert_eq!(cpu.apply_lsr_op(0x01), 0x00);
		assert_eq!(cpu.n, 0);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.z, 1);

		assert_eq!(cpu.apply_lsr_op(0x02), 0x01);
		assert_eq!(cpu.c, 0);
		assert_eq!(cpu.z, 0);
	}

	#[test]
	fn rol_op() {
		let mut cpu = Cpu::new();

		cpu.c = 1;
		assert_eq!(cpu.apply_rol_op(0b1001_0000), 0b0010_0001);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);
		assert_eq!(cpu.z, 0);

		cpu.c = 0;
		assert_eq!(cpu.apply_rol_op(0b0101_0100), 0b1010_1000);
		assert_eq!(cpu.c, 0);
		assert_eq!(cpu.n, 1);
		assert_eq!(cpu.z, 0);

		cpu.c = 0;
		assert_eq!(cpu.apply_rol_op(0b1000_0000), 0x00);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);
		assert_eq!(cpu.z, 1);
	}

	#[test]
	fn ror_op() {
		let mut cpu = Cpu::new();

		cpu.c = 1;
		assert_eq!(cpu.apply_ror_op(0b1001_0000), 0b1100_1000);
		assert_eq!(cpu.c, 0);
		assert_eq!(cpu.n, 1);
		assert_eq!(cpu.z, 0);

		cpu.c = 0;
		assert_eq!(cpu.apply_ror_op(0b0101_0101), 0b0010_1010);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);
		assert_eq!(cpu.z, 0);

		cpu.c = 0;
		assert_eq!(cpu.apply_ror_op(0b0000_0001), 0x00);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);
		assert_eq!(cpu.z, 1);
	}

	#[test]
	fn sbc_op() {
		// TODO: need more testing on flags
		let mut cpu = Cpu::new();

		cpu.c = 1;
		cpu.a = 0xFE;
		assert_eq!(cpu.apply_sbc_op(0x01), 0xFD);
		assert_eq!(cpu.v, 0);
	}
}