pub mod opcodes;

use std::io;
use std::process;

use crate::opcodes::OpCode as opc;

const NUM_OF_MEMORY_CELLS: usize = 65536;
const NUM_OF_REGISTERS: usize = 10;
const PRINTER_ADDR: usize = NUM_OF_MEMORY_CELLS - 3;

// Double width registers
const REG_A: u8 = 0xA;
const REG_B: u8 = 0xB;
const REG_C: u8 = 0xC;
const REG_D: u8 = 0xD;

fn is_double_reg(r: u8) -> bool {
    (REG_A..=REG_D).contains(&r)
}

fn reg_width(r: u8) -> u8 {
    if is_double_reg(r) {
        2
    } else {
        1
    }
}

pub type Code = Vec<CodeSection>;

pub struct CodeSection {
    pub org: u16,
    pub code: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct VM {
    registers: Vec<u8>,
    memory: Vec<u8>,
    pc: u16,
    sp: u16,
    output: String,
    printer: String,
    debug_mode: bool,
}

macro_rules! simple_instr_imm {
    ($fnname:ident, $oper:tt) => {
        fn $fnname(&mut self, r: u8, data: u16) {
            let val = self.read_reg(r);
            self.write_reg(r, val $oper data);
        }
    };
}

macro_rules! simple_instr_addr {
    ($fnname:ident, $oper:tt) => {
        fn $fnname(&mut self, r: u8, addr: u16) {
            let data = self.read_mem(addr, reg_width(r));
            let val = self.read_reg(r);
            self.write_reg(r, val $oper data);
        }
    };
}

macro_rules! simple_instr_reg {
    ($fnname:ident, $oper:tt) => {
        fn $fnname(&mut self, dest: u8, src: u8) {
            let data1 = self.read_reg(src);
            let data2 = self.read_reg(dest);
            self.write_reg(dest, data1 $oper data2);
        }
    };
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
            debug_mode: false,
        }
    }

    pub fn install_code(&mut self, code: &[CodeSection]) {
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
        self.pc = ((u16::from(self.memory[0xFFFE])) << 8) | u16::from(self.memory[0xFFFF]);
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
        let b1 = u16::from(self.fetch_byte());
        let b2 = u16::from(self.fetch_byte());
        (b1 << 8) | b2
    }

    fn print_memory(&self) {
        print!("Memory   00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F  10 11 12 13 14 15 16 17 18 19 1A 1B 1C 1D 1E 1F\n\n");
        let mut i = 0;

        while i < 256 {
            print!("{:02X}", i);
            print!("       ");

            for j in 0..16 {
                print!("{:02X}", self.memory[i + j]);
                print!(" ");
            }
            print!(" ");

            for j in 16..32 {
                print!("{:02X}", self.memory[i + j]);
                print!(" ");
            }
            println!();
            i += 32
        }
    }

    fn print_registers(&self) {
        println!("0    1    2    3    4    5    6    7    8    9");
        for reg in &self.registers {
            print!("0x{:02X} ", reg);
        }
        println!("\nPC: 0x{:02X} | SP: 0x{:02X}", self.pc, self.sp);
    }

    pub fn run(&mut self) -> Result<(), &str> {
        macro_rules! instruction {
            ($inst:ident) => {{
                self.$inst();
            }};

            ($inst:ident, u8) => {{
                let arg1 = self.fetch_byte();
                self.$inst(arg1);
            }};

            ($inst:ident, u16) => {{
                let arg1 = self.fetch_u16();
                self.$inst(arg1);
            }};

            ($inst:ident, u8, u8) => {{
                let arg1 = self.fetch_byte();
                let arg2 = self.fetch_byte();
                self.$inst(arg1, arg2);
            }};

            ($inst:ident, u8, u16) => {{
                let arg1 = self.fetch_byte();
                let arg2 = self.fetch_u16();
                self.$inst(arg1, arg2);
            }};
        }

        let mut debug_disabled = false;

        loop {
            let opcode = opc::from(self.fetch_byte());

            if self.debug_mode {
                println!("Breakpoint hit at 0x{:02X}", self.pc - 1);

                loop {
                    print!("Debug> ");
                    io::Write::flush(&mut io::stdout()).expect("flush failed!");

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap_or_default();
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }

                    match parts[0] {
                        "step" => break,
                        "memory" | "mem" => {
                            if parts.len() == 1 {
                                self.print_memory();
                                continue;
                            }

                            let address = parts[1].parse::<u16>().unwrap_or_default();
                            println!(
                                "[{:02X}] = 0x{:02X}",
                                address, self.memory[address as usize]
                            );
                        }
                        "continue" | "con" => {
                            self.debug_mode = false;
                            break;
                        }
                        "disable" | "dis" => debug_disabled = true,
                        "enable" | "en" => debug_disabled = false,
                        "next" => println!("Next Instruction: {:?}", opcode),
                        "registers" | "reg" => self.print_registers(),
                        "printer" | "print" => println!("{}", self.printer),
                        "exit" | "quit" => process::exit(0),
                        _ => println!("Unknown command `{}`", parts[0]),
                    }
                }
            }

            match opcode {
                opc::LOADI => instruction!(inst_loadi, u8, u16),
                opc::LOADA => instruction!(inst_loada, u8, u16),
                opc::LOADR => instruction!(inst_loadr, u8, u8),

                opc::STRA => instruction!(inst_stra, u8, u16),
                opc::STRR => instruction!(inst_strr, u8, u8),

                opc::XFER => instruction!(inst_xfer, u8, u8),

                opc::ADDI => instruction!(inst_addi, u8, u16),
                opc::ADDA => instruction!(inst_adda, u8, u16),
                opc::ADDR => instruction!(inst_addr, u8, u8),

                opc::ORI => instruction!(inst_ori, u8, u16),
                opc::ORA => instruction!(inst_ora, u8, u16),
                opc::ORR => instruction!(inst_orr, u8, u8),

                opc::ANDI => instruction!(inst_andi, u8, u16),
                opc::ANDA => instruction!(inst_anda, u8, u16),
                opc::ANDR => instruction!(inst_andr, u8, u8),

                opc::XORI => instruction!(inst_xori, u8, u16),
                opc::XORA => instruction!(inst_xora, u8, u16),
                opc::XORR => instruction!(inst_xorr, u8, u8),

                opc::ROTR => instruction!(inst_rotr, u8, u8),
                opc::ROTL => instruction!(inst_rotl, u8, u8),

                opc::JMP => instruction!(inst_jmp, u8, u16),
                opc::JMPA => instruction!(inst_jmpa, u16),

                opc::HALT => break,

                opc::LDSPI => instruction!(inst_ldspi, u16),
                opc::LDSPA => instruction!(inst_ldspa, u16),
                opc::LDSPR => instruction!(inst_ldspr, u8),

                opc::PUSH => instruction!(inst_push, u8),
                opc::POP => instruction!(inst_pop, u8),

                opc::CALLA => instruction!(inst_calla, u16),
                opc::CALLR => instruction!(inst_callr, u8),

                opc::RTN => instruction!(inst_rtn),

                opc::DEBUG => {
                    if !debug_disabled {
                        self.debug_mode = true
                    }
                }
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
            u16::from(self.read_single_reg(r))
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
        self.registers[r as usize]
    }

    fn write_single_reg(&mut self, r: u8, data: u8) {
        self.registers[r as usize] = data;
    }

    fn read_double_reg(&self, r: u8) -> u16 {
        if r == REG_A {
            (u16::from(self.registers[2]) << 8) | u16::from(self.registers[3])
        } else if r == REG_B {
            (u16::from(self.registers[4]) << 8) | u16::from(self.registers[5])
        } else if r == REG_C {
            (u16::from(self.registers[6]) << 8) | u16::from(self.registers[7])
        } else if r == REG_D {
            (u16::from(self.registers[8]) << 8) | u16::from(self.registers[9])
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
            return u16::from(self.memory[addr as usize]);
        } else if width == 2 {
            let b1 = u16::from(self.memory[addr as usize]);
            let b2 = u16::from(self.memory[(addr + 1) as usize]);
            return (b1 << 8) | b2;
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
        self.write_mem(addr, 1, u16::from(data))
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
        let data = self.read_mem(addr, reg_width(r));
        self.write_reg(r, data);
    }

    fn inst_loadr(&mut self, dest: u8, src: u8) {
        let addr = self.read_reg(src);
        self.inst_loada(dest, addr);
    }

    // STORE
    fn inst_stra(&mut self, src: u8, addr: u16) {
        let data = self.read_reg(src);
        self.write_mem(addr, reg_width(src), data);
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

    // ADD
    simple_instr_imm!(inst_addi, +);
    simple_instr_addr!(inst_adda, +);
    simple_instr_reg!(inst_addr, +);

    // OR
    simple_instr_imm!(inst_ori, |);
    simple_instr_addr!(inst_ora, |);
    simple_instr_reg!(inst_orr, |);

    // AND
    simple_instr_imm!(inst_andi, &);
    simple_instr_addr!(inst_anda, &);
    simple_instr_reg!(inst_andr, &);

    // XOR
    simple_instr_imm!(inst_xori, ^);
    simple_instr_addr!(inst_xora, ^);
    simple_instr_reg!(inst_xorr, ^);

    // ROTATE
    fn inst_rotr(&mut self, dest: u8, places: u8) {
        if is_double_reg(dest) {
            let val = self.read_double_reg(dest);
            let data = val.rotate_right(u32::from(places));
            self.write_double_reg(dest, data);
        }
    }

    fn inst_rotl(&mut self, dest: u8, places: u8) {
        if is_double_reg(dest) {
            let val = self.read_double_reg(dest);
            let data = val.rotate_left(u32::from(places));
            self.write_double_reg(dest, data);
        }
    }

    // JUMP
    fn inst_jmp(&mut self, r: u8, pc: u16) {
        let zero_reg = self.read_reg(0);
        let check_reg = self.read_reg(r);
        if check_reg == zero_reg {
            self.pc = pc;
        }
    }

    fn inst_jmpa(&mut self, pc: u16) {
        self.pc = pc;
    }

    // LOAD SP
    fn inst_ldspi(&mut self, addr: u16) {
        self.sp = addr;
    }

    fn inst_ldspa(&mut self, addr: u16) {
        let sp_val = self.read_mem_u16(addr);
        self.sp = sp_val;
    }

    fn inst_ldspr(&mut self, r: u8) {
        let sp_val = self.read_reg(r);
        self.sp = sp_val;
    }

    // PUSH/POP
    fn push_u16(&mut self, data: u16) {
        self.sp -= 2;
        let sp = self.sp;
        self.write_mem_u16(sp, data);
    }

    fn push_u8(&mut self, data: u8) {
        self.sp -= 1;
        let sp = self.sp;
        self.write_mem_u8(sp, data);
    }

    fn pop_u16(&mut self) -> u16 {
        let sp = self.sp;
        let data = self.read_mem_u16(sp);
        self.sp += 2;
        data
    }

    fn pop_u8(&mut self) -> u8 {
        let sp = self.sp;
        let data = self.read_mem_u8(sp);
        self.sp += 1;
        data
    }

    fn inst_push(&mut self, r: u8) {
        if is_double_reg(r) {
            let val = self.read_double_reg(r);
            self.push_u16(val);
        } else {
            let val = self.read_single_reg(r);
            self.push_u8(val);
        }
    }

    fn inst_pop(&mut self, r: u8) {
        if is_double_reg(r) {
            let val = self.pop_u16();
            self.write_double_reg(r, val);
        } else {
            let val = self.pop_u8();
            self.write_single_reg(r, val);
        }
    }

    // CALL
    fn inst_calla(&mut self, addr: u16) {
        let pc = self.pc;
        self.push_u16(pc);
        self.pc = addr;
    }

    fn inst_callr(&mut self, r: u8) {
        let pc = self.pc;
        self.push_u16(pc);

        let new_pc = self.read_reg(r);
        self.pc = new_pc;
    }

    fn inst_rtn(&mut self) {
        let pc = self.pop_u16();
        self.pc = pc;
    }
}
