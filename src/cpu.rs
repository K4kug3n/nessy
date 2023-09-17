use crate::memory::Memory;

pub struct Cpu {
	pc: u16,
	sp: u8,

	p: u8,
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
}

// enum AddrMode {
// 	Immediate,
// 	ZeroPage,
// 	ZeroPageX,
// 	Absolute,
// 	AbsoluteX,
// 	AbsoluteY,
// 	IndirectX,
// 	IndirectY
// }

enum Instruction {
	Adc(u8),
	AslA,
	Asl(u16),
	Nop
}

impl Cpu {
	pub fn new() -> Cpu {
		Cpu {
			pc: 0,
			sp: 255,

			p: 0,
			a: 0,
			x: 0,
			y: 0,

			n: 0,
			v: 0,
			b: 0,
			d: 0,
			i: 0,
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

	fn fetch_absolute_adress(&mut self, memory: &Memory) -> u16 {
		// Little endian
		u16::from(self.fetch(memory)) + (u16::from(self.fetch(memory)) << 8)
	}

	fn decode(&mut self, memory: &Memory, opcode: u8) -> Instruction {
		match opcode {
			0x69 => Instruction::Adc(self.fetch(memory)),
			0x65 => {
				let adress = u16::from(self.fetch(memory));
				Instruction::Adc(memory.read(adress))
			},
			0x75 => {
				Instruction::Adc(memory.read(u16::from(self.fetch(memory)) + u16::from(self.x)))
			},
			0x6D => {
				let adress = self.fetch_absolute_adress(memory);
				Instruction::Adc(memory.read(adress))
			},
			0x7D => {
				let adress = self.fetch_absolute_adress(memory);
				Instruction::Adc(memory.read(adress + u16::from(self.x)))
			},
			0x79 => {
				let adress = self.fetch_absolute_adress(memory);
				Instruction::Adc(memory.read(adress + u16::from(self.y)))
			},
			0x61 => {
				let adress = (u16::from(self.fetch(memory)) + u16::from(self.x)) & 0x00ff;
				Instruction::Adc(memory.read(adress))
			},
			0x71 => {
				let indirect = u16::from(self.fetch(memory));
				let adress = u16::from(memory.read(indirect)) + (u16::from(memory.read(indirect+1)) << 8);
				Instruction::Adc(memory.read(adress + u16::from(self.y)))
			},
			
			0x0A => {
				Instruction::AslA
			},
			0x0E => {
				let adress = self.fetch_absolute_adress(memory);
				Instruction::Asl(adress)
			},
			0x1E => {
				let adress = self.fetch_absolute_adress(memory);
				Instruction::Asl(adress + u16::from(self.x))
			},
			0x06 => {
				let adress = u16::from(self.fetch(memory));
				Instruction::Asl(adress)
			},
			0x16 => {
				let adress = (u16::from(self.fetch(memory)) + u16::from(self.x)) & 0x00ff;
				Instruction::Asl(adress)
			},

			_ => Instruction::Nop
		}
	}

	fn execute(&mut self, memory: &mut Memory, instruction: Instruction) {
		match instruction {
			Instruction::Adc(value) => {
				self.a = self.apply_adc_op(value);
			},
			Instruction::AslA => {
				self.a = self.apply_asl_op(self.a);
			},
			Instruction::Asl(adress) => {
				let value = memory.read(adress);
				
				memory.write(adress, self.apply_asl_op(value));
			},
			Instruction::Nop => {}
		}
	}

	fn apply_adc_op(&mut self, value: u8) -> u8 {
		let (temp, overflowed_1) = u8::overflowing_add(value, value);
		let (result, overflowed_2) = u8::overflowing_add(temp, self.c);
		
		self.c = if overflowed_1 || overflowed_2 { 1 } else { 0 };
		self.v = if (value & 0x80) != (result & 0x80) { 1 } else { 0 };
		self.n = if result & 0x80 == 0x80 { 1 } else { 0 };
		self.z = if result == 0 { 1 } else { 0 };
		// TODO: p if page crossed
		
		result
	}

	fn apply_asl_op(&mut self, value: u8) -> u8 {
		self.c = (value & 0x80) >> 7;

		let result = (value & 0x7F) << 1;

		self.n = (result & 0x80) >> 7;
		self.z = if result == 0 { 1 } else { 0 };

		result
	}
}