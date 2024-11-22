use core::panic;
use std::fmt;

use crate::memory::Memory;

pub struct Cpu {
	pub pc: u16,
	sp: u8,

	// Registers
	a: u8,
	x: u8,
	y: u8,

	// Flags
	n: u8,
	v: u8,
	b: u8,
	d: u8,
	i: u8,
	z: u8,
	c: u8,

	extra_cycle: u8
}

#[derive(Debug)]
enum Instruction {
	Adc,
	And,
	Asl,
	Bcc,
	Bcs,
	Beq,
	Bit,
	Bmi,
	Bne,
	Bpl,
	Brk,
	Bvc,
	Bvs,
	Clc,
	Cld,
	Cli,
	Clv,
	Cmp,
	Cpx,
	Cpy,
	Dec,
	Dex,
	Dey,
	Eor,
	Inc,
	Inx,
	Iny,
	Jmp,
	Jsr,
	Lda,
	Ldx,
	Ldy,
	Lsr,
	Nop,
	Ora,
	Pha,
	Php,
	Pla,
	Plp,
	Rol,
	Ror,
	Rti,
	Rts,
	Sbc,
	Sec,
	Sed,
	Sei,
	Sta,
	Stx,
	Sty,
	Tax,
	Tay,
	Tsx,
	Txa,
	Txs,
	Tya
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Debug)]
enum AddrMode {
	Immediate,
	Accumulator,
	Absolute,
	XIndexedAbsolute,
	YIndexedAbsolute,
	AbsoluteIndirect,
	ZeroPage,
	XIndexedZeroPage,
	YIndexedZeroPage,
	XIndexedZeroPageIndirect,
	ZeroPageIndirectYIndexed,
	Relative,
	None
}

impl Cpu {
	pub fn new() -> Cpu {
		Cpu {
			pc: 0x00,
			sp: 0xFD,

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

	pub fn reset(&mut self, memory: &mut Memory) {
		self.sp = 0xFD;
		self.set_status(0b100100);

		self.pc = memory.read_u16(0xFFFC);
	}

	pub fn run(&mut self, memory: &mut Memory)
	{
		self.run_with_callback(memory, |_, _|{});
	}

	pub fn run_with_callback<F>(&mut self, memory: &mut Memory, mut callback: F) 
	where 
		F: FnMut(&mut Cpu, &Memory),
	{
		loop {
			callback(self, &memory);

			let opcode = self.fetch(memory);

			let (instr, addr_mode, _, _) = self.decode(opcode);
			if let Instruction::Brk = instr {
				break;
			}

			self.extra_cycle = 0;
			self.execute(memory, &instr, &addr_mode);
		}
	}

	#[allow(dead_code)]
	pub fn load_and_run(&mut self, memory: &mut Memory, pgr: &Vec<u8>) {
		for i in 0..(pgr.len() as u16) {
			memory.write(0x0200 + i, pgr[i as usize]);
		}

		self.reset(memory);
		self.pc = 0x0200;

		self.run(memory);
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

	fn get_status(&self) -> u8 {
		(self.n << 7) + (self.v << 6) + (1 << 5) + (self.b << 4) + (self.d << 3) + (self.i << 2) + (self.z << 1) + self.c
	}

	fn is_crossing(origin: u16, next: u16) -> bool {
		(origin & 0xFF00) != (next & 0xFF00)
	}

	fn fetch(&mut self, memory: &Memory) -> u8 {
		let value = memory.read(self.pc);
		self.pc += 1;
		value
	}

	fn fetch_relative(&mut self, memory: &Memory) -> u16 {
		let value = self.fetch(memory);

		let mut offset = i32::from(value);
		if value >> 7 != 0 { // is a negative
			offset -= 256
		}

		u16::try_from(i32::from(self.pc) + offset).unwrap()
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
		let adress = absolute.wrapping_add(self.x as u16);

		self.extra_cycle = u8::from(Cpu::is_crossing(absolute, adress));

		adress
	}

	fn fetch_y_indexed_absolute_adress(&mut self, memory: &Memory) -> u16 {
		let absolute = self.fetch_absolute_adress(memory);
		let adress = absolute.wrapping_add(self.y as u16);

		self.extra_cycle = u8::from(Cpu::is_crossing(absolute, adress));

		adress
	}

	fn fetch_zero_page_adress(&mut self, memory: &Memory) -> u16 {
		u16::from(self.fetch(memory))
	}

	fn fetch_x_indexed_zero_page_adress(&mut self, memory: &Memory) -> u16 {
		self.fetch(memory).wrapping_add(self.x) as u16
	}

	fn fetch_y_indexed_zero_page_adress(&mut self, memory: &Memory) -> u16 {
		self.fetch(memory).wrapping_add(self.y) as u16
	}

	fn fetch_x_indexed_zero_page_indirect_adress(&mut self, memory: &Memory) -> u16 {
		let indirect = self.fetch(memory).wrapping_add(self.x);
		
		// Next memory loc must be on zero page
		// Little endian in memory
		(u16::from(memory.read(indirect.wrapping_add(1) as u16)) << 8) | u16::from(memory.read(indirect as u16))
	}

	fn fetch_zero_page_indirect_y_indexed_adress(&mut self, memory: &Memory) -> u16 {
		let pointer = self.fetch(memory);

		// Little endian
		let lo = memory.read(pointer as u16) as u16;
		let hi = memory.read(pointer.wrapping_add(1) as u16) as u16;
		let indirect = lo | (hi << 8);
		let adress = indirect.wrapping_add(self.y as u16);

		self.extra_cycle = u8::from(Cpu::is_crossing(indirect, adress)); // is_crossing

		adress
	}

	fn decode(&mut self, opcode: u8) -> (Instruction, AddrMode, u8, u8) {
		match opcode {
			0x69 => (Instruction::Adc, AddrMode::Immediate, 2, 2),
			0x6D => (Instruction::Adc, AddrMode::Absolute, 3, 4),
			0x7D => (Instruction::Adc, AddrMode::XIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0x79 => (Instruction::Adc, AddrMode::YIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0x65 => (Instruction::Adc, AddrMode::ZeroPage, 2, 3),
			0x75 => (Instruction::Adc, AddrMode::XIndexedZeroPage, 2, 4),
			0x61 => (Instruction::Adc, AddrMode::XIndexedZeroPageIndirect, 2, 6),
			0x71 => (Instruction::Adc, AddrMode::ZeroPageIndirectYIndexed, 2, 5 /* + self.extra_cycle */),
			
			0x29 => (Instruction::And, AddrMode::Immediate, 2, 2),
			0x2D => (Instruction::And, AddrMode::Absolute, 3, 4),
			0x3D => (Instruction::And, AddrMode::XIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0x39 => (Instruction::And, AddrMode::YIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0x25 => (Instruction::And, AddrMode::ZeroPage, 2, 3),
			0x35 => (Instruction::And, AddrMode::XIndexedZeroPage, 2, 4),
			0x21 => (Instruction::And, AddrMode::XIndexedZeroPageIndirect, 2, 6),
			0x31 => (Instruction::And, AddrMode::ZeroPageIndirectYIndexed, 2, 5 /* + self.extra_cycle */),

			0x0A => (Instruction::Asl, AddrMode::Accumulator, 1, 2),
			0x0E => (Instruction::Asl, AddrMode::Absolute, 3, 6),
			0x1E => (Instruction::Asl, AddrMode::XIndexedAbsolute, 3, 7),
			0x06 => (Instruction::Asl, AddrMode::ZeroPage, 2, 5),
			0x16 => (Instruction::Asl, AddrMode::XIndexedZeroPage, 2, 6),

			0x90 => (Instruction::Bcc, AddrMode::Relative, 2, 2 /* + p + t */),
			0xB0 => (Instruction::Bcs, AddrMode::Relative, 2, 2 /* + p + t */),
			0xF0 => (Instruction::Beq, AddrMode::Relative, 2, 2 /* + p + t */),

			0x2C => (Instruction::Bit, AddrMode::Absolute, 3, 4),
			0x24 => (Instruction::Bit, AddrMode::ZeroPage, 2, 3),

			0x30 => (Instruction::Bmi, AddrMode::Relative, 2, 2 /* + p + t */),
			0xD0 => (Instruction::Bne, AddrMode::Relative, 2, 2 /* + p + t */),
			0x10 => (Instruction::Bpl, AddrMode::Relative, 2, 2 /* + p + t */),

			0x00 => (Instruction::Brk, AddrMode::None, 1, 7),

			0x50 => (Instruction::Bvc, AddrMode::Relative, 2, 2 /* + p + t */),
			0x70 => (Instruction::Bvs, AddrMode::Relative, 2, 2 /* + p + t */),

			0x18 => (Instruction::Clc, AddrMode::None,1, 2),
			0xD8 => (Instruction::Cld, AddrMode::None, 1, 2),
			0x58 => (Instruction::Cli, AddrMode::None, 1, 2),
			0xB8 => (Instruction::Clv, AddrMode::None, 1, 2),

			0xC9 => (Instruction::Cmp, AddrMode::Immediate, 2, 2),
			0xCD => (Instruction::Cmp, AddrMode::Absolute, 3, 4),
			0xDD => (Instruction::Cmp, AddrMode::XIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0xD9 => (Instruction::Cmp, AddrMode::YIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0xC5 => (Instruction::Cmp, AddrMode::ZeroPage, 2, 3),
			0xD5 => (Instruction::Cmp, AddrMode::XIndexedZeroPage, 2, 4),
			0xC1 => (Instruction::Cmp, AddrMode::XIndexedZeroPageIndirect, 2, 6),
			0xD1 => (Instruction::Cmp, AddrMode::ZeroPageIndirectYIndexed, 2, 5 /* + self.extra_cycle */),

			0xE0 => (Instruction::Cpx, AddrMode::Immediate, 2, 2),
			0xEC => (Instruction::Cpx, AddrMode::Absolute, 3, 4),
			0xE4 => (Instruction::Cpx, AddrMode::ZeroPage, 2, 3),

			0xC0 => (Instruction::Cpy, AddrMode::Immediate, 2, 2),
			0xCC => (Instruction::Cpy, AddrMode::Absolute, 3, 4),
			0xC4 => (Instruction::Cpy, AddrMode::ZeroPage, 2, 3),

			0xCE => (Instruction::Dec, AddrMode::Absolute, 3, 6),
			0xDE => (Instruction::Dec, AddrMode::XIndexedAbsolute, 3, 7),
			0xC6 => (Instruction::Dec, AddrMode::ZeroPage, 2, 5),
			0xD6 => (Instruction::Dec, AddrMode::XIndexedZeroPage, 2, 6),

			0xCA => (Instruction::Dex, AddrMode::None, 1, 2),
			0x88 => (Instruction::Dey, AddrMode::None, 1, 2),

			0x49 => (Instruction::Eor, AddrMode::Immediate, 2, 2),
			0x4D => (Instruction::Eor, AddrMode::Absolute, 3, 4),
			0x5D => (Instruction::Eor, AddrMode::XIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0x59 => (Instruction::Eor, AddrMode::YIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0x45 => (Instruction::Eor, AddrMode::ZeroPage, 2, 3),
			0x55 => (Instruction::Eor, AddrMode::XIndexedZeroPage, 2, 4),
			0x41 => (Instruction::Eor, AddrMode::XIndexedZeroPageIndirect, 2, 6),
			0x51 => (Instruction::Eor, AddrMode::ZeroPageIndirectYIndexed, 2, 5 /* + self.extra_cycle */),

			0xEE => (Instruction::Inc, AddrMode::Absolute, 3, 6),
			0xFE => (Instruction::Inc, AddrMode::XIndexedAbsolute, 3, 7),
			0xE6 => (Instruction::Inc, AddrMode::ZeroPage, 2, 5),
			0xF6 => (Instruction::Inc, AddrMode::XIndexedZeroPage, 2, 6),

			0xE8 => (Instruction::Inx, AddrMode::None, 1, 2),
			0xC8 => (Instruction::Iny, AddrMode::None, 1, 2),

			0x4C => (Instruction::Jmp, AddrMode::Absolute, 3, 3),
			0x6C => (Instruction::Jmp, AddrMode::AbsoluteIndirect, 3, 5),

			0x20 => (Instruction::Jsr, AddrMode::Absolute, 3, 6),

			0xA9 => (Instruction::Lda, AddrMode::Immediate, 2, 2),
			0xAD => (Instruction::Lda, AddrMode::Absolute, 3, 4),
			0xBD => (Instruction::Lda, AddrMode::XIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0xB9 => (Instruction::Lda, AddrMode::YIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0xA5 => (Instruction::Lda, AddrMode::ZeroPage, 2, 3),
			0xB5 => (Instruction::Lda, AddrMode::XIndexedZeroPage, 2, 4),
			0xA1 => (Instruction::Lda, AddrMode::XIndexedZeroPageIndirect, 2, 6),
			0xB1 => (Instruction::Lda, AddrMode::ZeroPageIndirectYIndexed, 2, 5 /* + self.extra_cycle */),

			0xA2 => (Instruction::Ldx, AddrMode::Immediate, 2, 2),
			0xAE => (Instruction::Ldx, AddrMode::Absolute, 3, 4),
			0xBE => (Instruction::Ldx, AddrMode::YIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0xA6 => (Instruction::Ldx, AddrMode::ZeroPage, 2, 3),
			0xB6 => (Instruction::Ldx, AddrMode::YIndexedZeroPage, 2, 4),

			0xA0 => (Instruction::Ldy, AddrMode::Immediate, 2, 2),
			0xAC => (Instruction::Ldy, AddrMode::Absolute, 3, 4),
			0xBC => (Instruction::Ldy, AddrMode::XIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0xA4 => (Instruction::Ldy, AddrMode::ZeroPage, 2, 3),
			0xB4 => (Instruction::Ldy, AddrMode::XIndexedZeroPage, 2, 4),

			0x4A => (Instruction::Lsr, AddrMode::Accumulator, 1, 2),
			0x4E => (Instruction::Lsr, AddrMode::Absolute, 3, 6),
			0x5E => (Instruction::Lsr, AddrMode::XIndexedAbsolute, 3, 7),
			0x46 => (Instruction::Lsr, AddrMode::ZeroPage, 2, 5),
			0x56 => (Instruction::Lsr, AddrMode::XIndexedZeroPage, 2, 6),

			0xEA => (Instruction::Nop, AddrMode::None, 1, 2),

			0x09 => (Instruction::Ora, AddrMode::Immediate, 2, 2),
			0x0D => (Instruction::Ora, AddrMode::Absolute, 3, 4),
			0x1D => (Instruction::Ora, AddrMode::XIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0x19 => (Instruction::Ora, AddrMode::YIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0x05 => (Instruction::Ora, AddrMode::ZeroPage, 2, 3),
			0x15 => (Instruction::Ora, AddrMode::XIndexedZeroPage, 2, 4),
			0x01 => (Instruction::Ora, AddrMode::XIndexedZeroPageIndirect, 2, 6),
			0x11 => (Instruction::Ora, AddrMode::ZeroPageIndirectYIndexed, 2, 5 /* + self.extra_cycle */),

			0x48 => (Instruction::Pha, AddrMode::None, 1, 3),
			0x08 => (Instruction::Php, AddrMode::None, 1, 3),
			0x68 => (Instruction::Pla, AddrMode::None, 1, 4),
			0x28 => (Instruction::Plp, AddrMode::None, 1, 4),

			0x2A => (Instruction::Rol, AddrMode::Accumulator, 1, 2),
			0x2E => (Instruction::Rol, AddrMode::Absolute, 3, 6),
			0x3E => (Instruction::Rol, AddrMode::XIndexedAbsolute, 3, 7),
			0x26 => (Instruction::Rol, AddrMode::ZeroPage, 2, 5),
			0x36 => (Instruction::Rol, AddrMode::XIndexedZeroPage, 2, 6),
			
			0x6A => (Instruction::Ror, AddrMode::Accumulator, 1, 2),
			0x6E => (Instruction::Ror, AddrMode::Absolute, 3, 6),
			0x7E => (Instruction::Ror, AddrMode::XIndexedAbsolute, 3, 7),
			0x66 => (Instruction::Ror, AddrMode::ZeroPage, 2, 5),
			0x76 => (Instruction::Ror, AddrMode::XIndexedZeroPage, 2, 6),

			0x40 => (Instruction::Rti, AddrMode::None, 1, 6),
			0x60 => (Instruction::Rts, AddrMode::None, 1, 6),

			0xE9 => (Instruction::Sbc, AddrMode::Immediate, 2, 2),
			0xED => (Instruction::Sbc, AddrMode::Absolute, 3, 4),
			0xFD => (Instruction::Sbc, AddrMode::XIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0xF9 => (Instruction::Sbc, AddrMode::YIndexedAbsolute, 3, 4 /* + self.extra_cycle */),
			0xE5 => (Instruction::Sbc, AddrMode::ZeroPage, 2, 3),
			0xF5 => (Instruction::Sbc, AddrMode::XIndexedZeroPage, 2, 4),
			0xE1 => (Instruction::Sbc, AddrMode::XIndexedZeroPageIndirect, 2, 6),
			0xF1 => (Instruction::Sbc, AddrMode::ZeroPageIndirectYIndexed, 2, 5 /* + self.extra_cycle */),

			0x38 => (Instruction::Sec, AddrMode::None, 1, 2),
			0xF8 => (Instruction::Sed, AddrMode::None, 1, 2),
			0x78 => (Instruction::Sei, AddrMode::None, 1, 2),

			0x8D => (Instruction::Sta, AddrMode::Absolute, 3, 4),
			0x9D => (Instruction::Sta, AddrMode::XIndexedAbsolute, 3, 5),
			0x99 => (Instruction::Sta, AddrMode::YIndexedAbsolute, 3, 5),
			0x85 => (Instruction::Sta, AddrMode::ZeroPage, 2, 3),
			0x95 => (Instruction::Sta, AddrMode::XIndexedZeroPage, 2, 4),
			0x81 => (Instruction::Sta, AddrMode::XIndexedZeroPageIndirect, 2, 6),
			0x91 => (Instruction::Sta, AddrMode::ZeroPageIndirectYIndexed, 2, 6),

			0x8E => (Instruction::Stx, AddrMode::Absolute, 3, 4),
			0x86 => (Instruction::Stx, AddrMode::ZeroPage, 2, 3),
			0x96 => (Instruction::Stx, AddrMode::YIndexedZeroPage, 2, 4),

			0x8C => (Instruction::Sty, AddrMode::Absolute, 3, 4),
			0x84 => (Instruction::Sty, AddrMode::ZeroPage, 2, 3),
			0x94 => (Instruction::Sty, AddrMode::XIndexedZeroPage, 2, 4),

			0xAA => (Instruction::Tax, AddrMode::None, 1, 2),
			0xA8 => (Instruction::Tay, AddrMode::None, 1, 2),
			0xBA => (Instruction::Tsx, AddrMode::None, 1, 2),
			0x8A => (Instruction::Txa, AddrMode::None, 1, 2),
			0x9A => (Instruction::Txs, AddrMode::None, 1, 2),
			0x98 => (Instruction::Tya, AddrMode::None, 1, 2),

			_ => {
				panic!("Opcode '{:#02x}' not implemented", opcode);
			}
		}
	}

	fn get_op_adress(&mut self, memory: &Memory, addr_mode: &AddrMode) -> u16 {
		match addr_mode {
			AddrMode::Immediate => {
				self.pc += 1; // Advance after the value
				self.pc - 1			
			},
			AddrMode::Absolute => self.fetch_absolute_adress(memory),
			AddrMode::XIndexedAbsolute => self.fetch_x_indexed_absolute_adress(memory),
			AddrMode::YIndexedAbsolute => self.fetch_y_indexed_absolute_adress(memory),
			AddrMode::AbsoluteIndirect => self.fetch_absolute_indirect_adress(memory),
			AddrMode::ZeroPage => self.fetch_zero_page_adress(memory),
			AddrMode::XIndexedZeroPage => self.fetch_x_indexed_zero_page_adress(memory),
			AddrMode::YIndexedZeroPage => self.fetch_y_indexed_zero_page_adress(memory),
			AddrMode::XIndexedZeroPageIndirect => self.fetch_x_indexed_zero_page_indirect_adress(memory),
			AddrMode::ZeroPageIndirectYIndexed => self.fetch_zero_page_indirect_y_indexed_adress(memory),
			AddrMode::Relative => self.fetch_relative(memory),
			_ => {
				panic!("Adress mode '{:?}' not usable to get adress", addr_mode);
			}
		}
	}

	fn execute(&mut self, memory: &mut Memory, instruction: &Instruction, addr_mode: &AddrMode) {
		match instruction {
			Instruction::Adc => self.apply_adc_op(memory, addr_mode),
			Instruction::And => self.apply_and_op(memory, addr_mode),
			Instruction::Asl => {
				if let AddrMode::Accumulator = addr_mode  {
					self.apply_asl_accumulator_op();
				}
				else {
					self.apply_asl_op(memory, addr_mode);
				}				
			},
			Instruction::Bcc => self.apply_branch(memory, self.c == 0),
			Instruction::Bcs => self.apply_branch(memory, self.c != 0),
			Instruction::Beq => self.apply_branch(memory, self.z != 0),
			Instruction::Bit => self.apply_bit_op(memory ,addr_mode),
			Instruction::Bmi => self.apply_branch(memory, self.n != 0),
			Instruction::Bne => self.apply_branch(memory, self.z == 0),
			Instruction::Bpl => self.apply_branch(memory, self.n == 0),
			Instruction::Brk => self.apply_brk_op(memory),
			Instruction::Bvc => self.apply_branch(memory, self.v == 0),
			Instruction::Bvs => self.apply_branch(memory, self.v != 0),
			Instruction::Clc => self.c = 0,
			Instruction::Cld => self.d = 0,
			Instruction::Cli => self.i = 0,
			Instruction::Clv => self.v = 0,
			Instruction::Cmp => self.apply_cmp_op( self.a, memory, addr_mode),
			Instruction::Cpx => self.apply_cmp_op( self.x, memory, addr_mode),
			Instruction::Cpy => self.apply_cmp_op( self.y, memory, addr_mode),
			Instruction::Dec => self.apply_dec_op(memory, addr_mode),
			Instruction::Dex => self.apply_dex_op(),
			Instruction::Dey => self.apply_dey_op(),
			Instruction::Eor => self.apply_eor_op(memory, addr_mode),
			Instruction::Inc => self.apply_inc_op(memory, addr_mode),
			Instruction::Inx => self.apply_inx_op(),
			Instruction::Iny => self.apply_iny_op(),
			Instruction::Jmp => self.pc = self.get_op_adress(memory, addr_mode),
			Instruction::Jsr => self.apply_jsr_op(memory, addr_mode),
			Instruction::Lda => self.a = self.apply_ld_op(memory, addr_mode),
			Instruction::Ldx => self.x = self.apply_ld_op(memory, addr_mode),
			Instruction::Ldy => self.y = self.apply_ld_op(memory, addr_mode),
			Instruction::Lsr => {
				if let AddrMode::Accumulator = addr_mode {
					self.apply_lsr_accumulator_op()
				}
				else {
					self.apply_lsr_op(memory, addr_mode);
				}
			},
			Instruction::Ora => self.apply_ora_op(memory, addr_mode),
			Instruction::Pha => self.apply_pha_op(memory),
			Instruction::Php => self.apply_php_op(memory),
			Instruction::Pla => self.apply_pla_op(memory),
			Instruction::Plp => self.apply_plp_op(memory),
			Instruction::Rol => {
				if let AddrMode::Accumulator = addr_mode {
					self.apply_rol_accumulator_op();
				}
				else {
					self.apply_rol_op(memory, addr_mode);
				}
			},
			Instruction::Ror => {
				if let AddrMode::Accumulator = addr_mode {
					self.apply_ror_accumulator_op();
				}
				else {
					self.apply_ror_op(memory, addr_mode);
				}
			},
			Instruction::Rti => self.apply_rti_op(memory),
			Instruction::Rts => self.apply_rts_op(memory),
			Instruction::Sbc => self.apply_sbc_op(memory, addr_mode),
			Instruction::Sec => self.c = 1,
			Instruction::Sed => self.d = 1,
			Instruction::Sei => self.i = 1,
			Instruction::Sta => memory.write(self.get_op_adress(memory, addr_mode), self.a),
			Instruction::Stx => memory.write(self.get_op_adress(memory, addr_mode), self.x),
			Instruction::Sty => memory.write(self.get_op_adress(memory, addr_mode), self.y),
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
			},
			Instruction::Tya => {
				self.a = self.y;
				self.z = u8::from(self.y == 0);
				self.n = self.y >> 7;
			},
			Instruction::Nop => {}
		}
	}

	fn apply_branch(&mut self, memory: &Memory, condition: bool) {
		let adress = self.fetch_relative(memory); // Advance the pc

		if condition {
			self.extra_cycle = 1 + u8::from(Cpu::is_crossing(self.pc, adress));

			self.pc = adress
		}
	}

	fn apply_adc_op(&mut self, memory: &Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);

		self.add_to_accumulator(value);
	}

	fn apply_and_op(&mut self, memory: &Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		let result = self.a & value;

		self.z = u8::from(result == 0);
		self.n = u8::from(result & 0x80 == 0x80);

		self.a = result;
	}

	fn apply_asl_accumulator_op(&mut self) {
		self.c = (self.a & 0x80) >> 7;

		let result = (self.a & 0x7F) << 1;

		self.n = result >> 7;
		self.z = u8::from(result == 0);

		self.a = result;
	}

	fn apply_asl_op(&mut self, memory: &mut Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		self.c = (value & 0x80) >> 7;

		let result = (value & 0x7F) << 1;

		self.n = result >> 7;
		self.z = u8::from(result == 0);

		memory.write(adress, result);
	}

	fn apply_bit_op(&mut self, memory: &Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		self.n = value >> 7;
		self.v = (value & 0x40) >> 6;

		self.z = u8::from((self.a & value) == 0);
	}

	fn apply_brk_op(&mut self, memory: &mut Memory) {
		self.pc += 2;
		let low_pc = u8::try_from(self.pc & 0x00FF).unwrap();
		let high_pc = u8::try_from((self.pc & 0xFF00) >> 8).unwrap();

		self.stack_push(memory, high_pc);
		self.stack_push(memory, low_pc);
		//let p = self.get_status();
		//self.stack_push(memory, p);

		self.pc = memory.read_u16(0xFFFE);
	}

	fn apply_cmp_op(&mut self, register: u8, memory: &Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		let (result, underflow) = register.overflowing_sub(value);
		self.z = u8::from(result == 0);
		self.n = result >> 7;
		self.c = u8::from(!underflow);
	}

	fn apply_dec_op(&mut self, memory: &mut Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		let result = value.wrapping_sub(1);

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		memory.write(adress, result);
	}

	fn apply_dex_op(&mut self) {
		let result = self.x.wrapping_sub(1);

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		self.x = result;
	}

	fn apply_dey_op(&mut self) {
		let result = self.y.wrapping_sub(1);

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		self.y = result;
	}

	fn apply_eor_op(&mut self, memory: &Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		let result = self.a ^ value;

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		self.a = result;
	}

	fn apply_inc_op(&mut self, memory: &mut Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		let (result, _) = value.overflowing_add(1);

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		memory.write(adress, result);
	}

	fn apply_inx_op(&mut self) {
		let (result, _) = self.x.overflowing_add(1);

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		self.x = result;
	}

	fn apply_iny_op(&mut self) {
		let (result, _) = self.y.overflowing_add(1);

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		self.y = result;
	}

	fn apply_jsr_op(&mut self, memory: &mut Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let low_pc = u8::try_from((self.pc - 1) & 0x00FF).unwrap();
		let high_pc = u8::try_from(((self.pc - 1) & 0xFF00) >> 8).unwrap();

		self.stack_push(memory, high_pc);
		self.stack_push(memory, low_pc);

		self.pc = adress;
	}

	fn apply_ld_op(&mut self, memory: &Memory, addr_mode: &AddrMode) -> u8 {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		self.z = u8::from(value == 0);
		self.n = value >> 7;

		value
	}

	fn apply_lsr_accumulator_op(&mut self) {
		self.c = self.a & 0x01;
		self.n = 0;

		let result = self.a >> 1;
		self.z = u8::from(result == 0);

		self.a = result;
	}

	fn apply_lsr_op(&mut self, memory: &mut Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		self.c = value & 0x01;
		self.n = 0;

		let result = value >> 1;
		self.z = u8::from(result == 0);

		memory.write(adress, result);
	}

	fn apply_ora_op(&mut self, memory: &Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		let result = value | self.a;

		self.z = u8::from(result == 0);
		self.n = result >> 7;

		self.a = result;
	}

	fn apply_pha_op(&mut self, memory: &mut Memory) {
		self.stack_push(memory, self.a);
	}

	fn apply_php_op(&mut self, memory: &mut Memory) {
		let p = self.get_status();
		
		self.stack_push(memory, p | 0b0001_0000); // Set B
	}

	fn apply_pla_op(&mut self, memory: &Memory) {
		self.a = self.stack_pop(memory);

		self.z = u8::from(self.a == 0);
		self.n = self.a >> 7;
	}

	fn apply_plp_op(&mut self, memory: &Memory) {
		let p = self.stack_pop(memory);

		self.set_status(p & 0b1110_1111); // Remove B
	}

	fn apply_rol_accumulator_op(&mut self) {
		let result = (self.a << 1) + self.c;
		self.c = self.a >> 7;
		self.n = (self.a & 0x40) >> 6;
		self.z = u8::from(result == 0);

		self.a = result;
	}

	fn apply_rol_op(&mut self, memory: &mut Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		let result = (value << 1) + self.c;
		self.c = value >> 7;
		self.n = (value & 0x40) >> 6;
		self.z = u8::from(result == 0);

		memory.write(adress, result);
	}

	fn apply_ror_accumulator_op(&mut self) {
		let result = (self.c << 7) + (self.a >> 1);
		self.n = self.c;
		self.c = self.a & 0x01;
		self.z = u8::from(result == 0);

		self.a = result;
	}

	fn apply_ror_op(&mut self, memory: &mut Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);
		let result = (self.c << 7) + (value >> 1);
		self.n = self.c;
		self.c = value & 0x01;
		self.z = u8::from(result == 0);

		memory.write(adress, result);
	}

	fn apply_rti_op(&mut self, memory: &Memory) {
		let p = self.stack_pop(memory);
		let low_pc = u16::from(self.stack_pop(memory));
		let high_pc = u16::from(self.stack_pop(memory));

		self.pc = (high_pc << 8) + low_pc;
		self.set_status(p);
	}

	fn apply_rts_op(&mut self, memory: &Memory) {
		let low_pc = u16::from(self.stack_pop(memory));
		let high_pc = u16::from(self.stack_pop(memory));

		self.pc = (high_pc << 8) + low_pc + 1;
	}

	fn apply_sbc_op(&mut self, memory: &Memory, addr_mode: &AddrMode) {
		let adress = self.get_op_adress(memory, addr_mode);
		let value = memory.read(adress);

		self.add_to_accumulator((value as i8).wrapping_neg().wrapping_sub(1) as u8);
	}

	fn add_to_accumulator(&mut self, value: u8) {
		let (temp, overflowed_1) = u8::overflowing_add(self.a, value);
		let (result, overflowed_2) = u8::overflowing_add(temp, self.c);
		
		self.c = u8::from(overflowed_1 || overflowed_2);
		self.v =  u8::from(!(((self.a ^ value) & 0x80) != 0) && (((self.a ^ result) & 0x80) != 0));
		self.n = result >> 7;
		self.z = u8::from(result == 0);
		
		self.a = result;
	}
}

pub fn trace(cpu: &mut Cpu, memory: &Memory) -> String {
	let pc = cpu.pc;
	
	let opcode = cpu.fetch(memory);

	let (instr, addr_mode, size, _) = cpu.decode(opcode);

	let mut hex_codes = vec![opcode];
	let asm_suffix = match size {
		1 => match addr_mode {
			AddrMode::Accumulator => String::from("A "),
			_ => String::from("")
		},
		2 => {
			let arg = memory.read(pc + 1);
			hex_codes.push(arg);

			let adress = cpu.get_op_adress(memory, &addr_mode);
			match addr_mode {
				AddrMode::Immediate => format!("#${:02x}", arg),
				AddrMode::ZeroPage => format!("${:02x} = {:02x}", arg, memory.read(adress)),
				AddrMode::XIndexedZeroPage => format!("${:02x},X @ {:02x} = {:02x}", arg, adress, memory.read(adress)),
				AddrMode::YIndexedZeroPage => format!("${:02x},Y @ {:02x} = {:02x}", arg, adress, memory.read(adress)),
				AddrMode::XIndexedZeroPageIndirect => format!("(${:02x},X) @ {:02x} = {:04x} = {:02x}", arg, cpu.x.wrapping_add(arg), adress, memory.read(adress)),
				AddrMode::ZeroPageIndirectYIndexed => {
					let lo = u16::from(memory.read(arg as u16));
					let hi = u16::from(memory.read(arg.wrapping_add(1) as u16));
					let indirect = lo + (hi << 8);
					format!("(${:02x}),Y = {:04x} @ {:04x} = {:02x}", arg, indirect, adress, memory.read(adress))
				},
				AddrMode::Relative =>  format!("${:04x}", adress),
				_ => panic!("Unexpected addressing mode {:?} with instruction's size {}", addr_mode, size)
			}
		},
		3 => {
			let lo_byte = memory.read(pc + 1);
			let hi_byte = memory.read(pc + 2);
			hex_codes.push(lo_byte);
			hex_codes.push(hi_byte);
			let arg = u16::from(lo_byte) + (u16::from(hi_byte) << 8);

			let adress = cpu.get_op_adress(memory, &addr_mode);
			match addr_mode {
				AddrMode::Absolute => match instr {
					Instruction::Jmp | Instruction::Jsr => format!("${:04x}", adress),
					_ => format!("${:04x} = {:02x}", adress, memory.read(adress))
				},
				AddrMode::XIndexedAbsolute => format!("${:04x},X @ {:04x} = {:02x}", arg, adress, memory.read(adress)),
				AddrMode::YIndexedAbsolute => format!("${:04x},Y @ {:04x} = {:02x}", arg, adress, memory.read(adress)),
				AddrMode::AbsoluteIndirect => format!("(${:04x}) = {:04x}", arg, adress),
				_ => panic!("Unexpected addressing mode {:?} with instruction's size {}", addr_mode, size)
			}
		},
		_ => panic!("Unexpected size of instruction: {}", size)
	};

	let hex_str = hex_codes.iter().map(|i| format!("{:02x}", i)).collect::<Vec<String>>().join(" ");
	let asm_str = format!("{} {}", instr, asm_suffix);

	cpu.pc = pc;

	format!("{:04x}  {:<8}  {:<30}  A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}", pc, hex_str, asm_str, cpu.a, cpu.x, cpu.y, cpu.get_status(), cpu.sp).to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
	use crate::mapper::test;

	use super::*;

	#[test]
	fn is_crossing() {
		assert_eq!(Cpu::is_crossing(0xABCD, 0xABCE), false);
		assert_eq!(Cpu::is_crossing(0x00FF, 0x0100), true);
		assert_eq!(Cpu::is_crossing(0xAB00, 0xFF00), true);
	}

	#[test]
    fn test_lda_immediate() {
        let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
		cpu.load_and_run(&mut memory, &vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.a, 5);
        assert!(cpu.get_status() & 0b0000_0010 == 0b00);
        assert!(cpu.get_status() & 0b1000_0000 == 0);
    }

	#[test]
    fn test_lda_absolute() {
        let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
		memory.write(0x0710, 0x55);

		cpu.load_and_run(&mut memory, &vec![0xad, 0x10, 0x07, 0x00]);
		
        assert_eq!(cpu.a, 0x55);
    }

	#[test]
    fn test_lda_zero_page() {
        let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
        memory.write(0x10, 0x55);

        cpu.load_and_run(&mut memory, &vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.a, 0x55);
    }

	#[test]
    fn test_tax() {
		let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
        cpu.a = 10;
        cpu.load_and_run(&mut memory,&vec![0xaa, 0x00]);

        assert_eq!(cpu.x, 10)
    }

	#[test]
	fn test_adc_x_indexed_zero_page() {
		// TODO: need more testing on flags
		let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
		
		memory.write(0x15, 0x20);
		cpu.x = 0x05;
		cpu.a = 0x01;
        // x indexed zero page
		cpu.load_and_run(&mut memory,&vec![0x75, 0x10, 0x00]);
		
		assert_eq!(cpu.a, 0x21);
		assert_eq!(cpu.c, 0);
	}

	#[test]
	fn test_cmp_immediate() {
		let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
		cpu.a = 0x10; // Set accumulator

		cpu.load_and_run(&mut memory,&vec![0xC9, 0x10, 0x00]);
		assert_eq!(cpu.z, 1);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);

		cpu.load_and_run(&mut memory,&vec![0xC9, 0x09, 0x00]);
		assert_eq!(cpu.z, 0);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);

		cpu.load_and_run(&mut memory,&vec![0xC9, 0x11, 0x00]);
		assert_eq!(cpu.z, 0);
		assert_eq!(cpu.c, 0);
		assert_eq!(cpu.n, 1);

		assert_eq!(cpu.a, 0x10);
	}

	#[test]
	fn test_lsr_accumulator() {
		let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
		
		cpu.a = 0x01;
		cpu.load_and_run(&mut memory,&vec![0x4A, 0x00]);
		assert_eq!(cpu.a, 0x00);
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.z, 1);
	}

	#[test]
	fn test_rol_absolute() {
		let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
		memory.write(0x0110, 0xA2); // 1010 0010

		cpu.load_and_run(&mut memory,&vec![0x2E, 0x10, 0x01, 0x00]);
		assert_eq!(memory.read(0x0110), 0x44); // 0100 0100
		assert_eq!(cpu.c, 1);
		assert_eq!(cpu.n, 0);
		assert_eq!(cpu.z, 0);
	}

	#[test]
	fn test_ror_absolute() {
		let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
		memory.write(0x0110, 0xA2); // 1010 0010

		cpu.load_and_run(&mut memory,&vec![0x6E, 0x10, 0x01, 0x00]);
		assert_eq!(memory.read(0x0110), 0x51); //  0101 0001
		assert_eq!(cpu.c, 0);
		assert_eq!(cpu.n, 0);
		assert_eq!(cpu.z, 0);
	}

	#[test]
    fn test_inx_overflow() {
        let mut cpu = Cpu::new();
		let mut memory = Memory::new(test::test_mapper());
        cpu.x = 0xff;
        cpu.load_and_run(&mut memory, &vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.x, 1)
    }

	#[test]
    fn test_lda_tax_inx() {
        let mut cpu = Cpu::new();
		// lda, tax, inx
		let mut memory = Memory::new(test::test_mapper());
        cpu.load_and_run(&mut memory, &vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.x, 0xc1)
    }

	#[test]
    fn test_status() {
		//  7 6 5 4 3 2 1 0
    	//  N V _ B D I Z C
    	//  | |   | | | | +--- Carry Flag
    	//  | |   | | | +----- Zero Flag
    	//  | |   | | +------- Interrupt Disable
    	//  | |   | +--------- Decimal Mode (not used on NES)
    	//  | |   +----------- Break Command
    	//  | +--------------- Overflow Flag
   		//  +----------------- Negative Flag
        let mut cpu = Cpu::new();
		cpu.set_status(0b0010_0100);

		assert_eq!(cpu.i, 1);
		assert_eq!(cpu.get_status(), 0b0010_0100);
    }
}