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
	Nop
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

			0xEA => Instruction::Nop,

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