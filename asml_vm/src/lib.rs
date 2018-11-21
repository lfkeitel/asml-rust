mod opcodes;

use opcodes::OpCode as opc;

const NUM_OF_MEMORY_CELLS: usize = 65536;
const NUM_OF_REGISTERS: usize = 10;
const PRINTER_ADDR: usize = NUM_OF_MEMORY_CELLS - 3;

// Double width registers
const REG_A: u8 = 0xA;
const REG_B: u8 = 0xB;
const REG_C: u8 = 0xC;
const REG_D: u8 = 0xD;

fn is_double_reg(r: u8) -> bool {
    r >= REG_A && r <= REG_D
}

pub type Code = Vec<CodeSection>;

pub struct CodeSection {
    pub org: u16,
    pub code: Vec<u8>,
}

#[derive(Debug)]
pub struct VM {
    registers: Vec<u8>,
    memory: Vec<u8>,
    pc: u16,
    sp: u16,
    output: String,
    printer: String,
    print_state: bool,
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: vec![0; NUM_OF_REGISTERS],
            memory: vec![0; NUM_OF_MEMORY_CELLS],
            pc: 0,
            sp: 0,
            output: String::with_capacity(20),
            printer: String::with_capacity(20),
            print_state: false,
        }
    }

    pub fn install_code(&mut self, code: &Code) {
        for section in code {
            let pc = section.org;

            for (i, b) in section.code.iter().enumerate() {
                let loc = i as u16 + pc;

                self.memory[loc as usize] = b.to_owned();
            }
        }

        self.reset();
    }

    pub fn reset(&mut self) {
        self.pc = (self.memory[0xFFFE] as u16) << 8 | self.memory[0xFFFF] as u16;
    }

    pub fn enable_print_state(&mut self) {
        self.print_state = true;
    }

    pub fn output(&self) -> String {
        self.output.clone()
    }

    fn fetch_byte(&mut self) -> u8 {
        let b = self.memory[self.pc as usize];
        self.pc += 1;
        b
    }

    fn fetch_u16(&mut self) -> u16 {
        let b1 = self.fetch_byte() as u16;
        let b2 = self.fetch_byte() as u16;
        return (b1 << 8) | b2;
    }

    fn print_state(&self) {}

    pub fn run(&mut self) -> Result<(), &str> {
        macro_rules! instruction {
            ($inst:ident, u8, u16) => {{
                let arg1 = self.fetch_byte();
                let arg2 = self.fetch_u16();
                self.$inst(arg1, arg2);
            }};

            ($inst:ident, u8, u8) => {{
                let arg1 = self.fetch_byte();
                let arg2 = self.fetch_byte();
                self.$inst(arg1, arg2);
            }};

            ($inst:ident, u16) => {{
                let arg1 = self.fetch_u16();
                self.$inst(arg1);
            }};

            ($inst:ident) => {{
                self.$inst();
            }};
        }

        loop {
            let opcode = opc::from(self.fetch_byte());

            if self.print_state {
                self.print_state();
                println!("{}", self.output);
                self.output.clear();
            }

            match opcode {
                opc::LOADI => instruction!(inst_loadi, u8, u16),
                opc::LOADA => instruction!(inst_loada, u8, u16),
                opc::LOADR => instruction!(inst_loadr, u8, u8),
                opc::STRA => instruction!(inst_stra, u8, u16),
                opc::STRR => instruction!(inst_strr, u8, u8),
                opc::XFER => instruction!(inst_xfer, u8, u8),
                opc::HALT => break,
                _ => return Err("Unknown opcode encountered"),
            }

            if self.memory[PRINTER_ADDR] > 0 {
                self.printer.push(self.memory[PRINTER_ADDR] as char);
                self.memory[PRINTER_ADDR] = 0
            }
        }

        self.output += self.printer.as_str();
        Ok(())
    }

    // Register manipulation
    fn read_reg(&self, r: u8) -> u16 {
        if is_double_reg(r) {
            self.read_double_reg(r)
        } else {
            self.read_single_reg(r) as u16
        }
    }

    fn write_reg(&mut self, r: u8, data: u16) {
        if is_double_reg(r) {
            self.write_double_reg(r, data);
        } else {
            self.write_single_reg(r, data as u8);
        }
    }

    fn read_single_reg(&self, r: u8) -> u8 {
        self.registers[r as usize] as u8
    }

    fn write_single_reg(&mut self, r: u8, data: u8) {
        self.registers[r as usize] = data;
    }

    fn read_double_reg(&self, r: u8) -> u16 {
        if r == REG_A {
            (self.registers[2] as u16) << 8 | self.registers[3] as u16
        } else if r == REG_B {
            (self.registers[4] as u16) << 8 | self.registers[5] as u16
        } else if r == REG_C {
            (self.registers[6] as u16) << 8 | self.registers[7] as u16
        } else if r == REG_D {
            (self.registers[8] as u16) << 8 | self.registers[9] as u16
        } else {
            0
        }
    }

    fn write_double_reg(&mut self, r: u8, data: u16) {
        if r == REG_A {
            self.registers[2] = (data >> 8) as u8;
            self.registers[3] = data as u8;
        } else if r == REG_B {
            self.registers[4] = (data >> 8) as u8;
            self.registers[5] = data as u8;
        } else if r == REG_C {
            self.registers[6] = (data >> 8) as u8;
            self.registers[7] = data as u8;
        } else if r == REG_D {
            self.registers[8] = (data >> 8) as u8;
            self.registers[9] = data as u8;
        }
    }

    // Memory manipulation
    fn read_mem(&self, addr: u16, width: u8) -> u16 {
        if width == 1 {
            return self.memory[addr as usize] as u16;
        } else if width == 2 {
            let b1 = self.memory[addr as usize] as u16;
            let b2 = self.memory[(addr + 1) as usize] as u16;
            return b1 << 8 | b2;
        }

        0
    }

    fn write_mem(&mut self, addr: u16, width: u8, data: u16) {
        if width == 1 {
            self.memory[addr as usize] = data as u8;
        } else if width == 2 {
            self.memory[addr as usize] = (data >> 8) as u8;
            self.memory[(addr + 1) as usize] = data as u8;
        }
    }

    fn read_mem_u8(&self, addr: u16) -> u8 {
        self.read_mem(addr, 1) as u8
    }

    fn write_mem_u8(&mut self, addr: u16, data: u8) {
        self.write_mem(addr, 1, data as u16)
    }

    fn read_mem_u16(&self, addr: u16) -> u16 {
        self.read_mem(addr, 2)
    }

    fn write_mem_u16(&mut self, addr: u16, data: u16) {
        self.write_mem(addr, 2, data)
    }

    // Instructions

    // LOAD
    fn inst_loadi(&mut self, r: u8, data: u16) {
        self.write_reg(r, data);
    }

    fn inst_loada(&mut self, r: u8, addr: u16) {
        if is_double_reg(r) {
            let data = self.read_mem_u16(addr);
            self.write_double_reg(r, data);
        } else {
            let data = self.read_mem_u8(addr);
            self.write_single_reg(r, data);
        }
    }

    fn inst_loadr(&mut self, dest: u8, src: u8) {
        let addr = self.read_reg(src);
        self.inst_loada(dest, addr);
    }

    // STORE
    fn inst_stra(&mut self, src: u8, addr: u16) {
        if is_double_reg(src) {
            let data = self.read_double_reg(src);
            self.write_mem_u16(addr, data);
        } else {
            let data = self.read_single_reg(src);
            self.write_mem_u8(addr, data);
        }
    }

    fn inst_strr(&mut self, src: u8, dest: u8) {
        let addr = self.read_reg(src);
        self.inst_stra(dest, addr);
    }

    // XFER
    fn inst_xfer(&mut self, dest: u8, src: u8) {
        let data = self.read_reg(src);
        self.write_reg(dest, data);
    }
}
