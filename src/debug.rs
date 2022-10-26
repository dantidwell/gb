use crate::cpu;
use core::panic;
use std::fmt;

enum JmpCondition {
    NonZero,
}

impl fmt::Display for JmpCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            JmpCondition::NonZero => "NZ",
        };
        write!(f, "${}", symbol)
    }
}

enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl fmt::Display for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Reg8::A => "A",
            Reg8::B => "B",
            Reg8::C => "C",
            Reg8::D => "D",
            Reg8::E => "E",
            Reg8::H => "H",
            Reg8::L => "L",
        };
        write!(f, "{}", symbol)
    }
}

enum Reg16 {
    BC,
    DE,
    HL,
    SP,
}

struct Imm8(u8);

impl fmt::Display for Imm8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:02x}", self.0)
    }
}

struct Imm16(u8, u8);

impl fmt::Display for Imm16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:02x}{:02x}", self.1, self.0)
    }
}

impl fmt::Display for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Reg16::BC => "BC",
            Reg16::DE => "DE",
            Reg16::HL => "HL",
            Reg16::SP => "SP",
        };
        write!(f, "{}", symbol)
    }
}

pub fn disassemble(stream: &mut [u8]) {
    let mut offset: usize = 0;
    loop {
        let soff = &stream[offset..];

        let instruction = cpu::Instruction::decode(soff[0]);
        let (s, n) = match instruction {
            cpu::Instruction::Call => disasm_call(soff),
            cpu::Instruction::CBPrefix => disasm_cb_prefixed(soff),
            cpu::Instruction::JmpRelIfNonZero => disasm_jmpr(soff, JmpCondition::NonZero),
            cpu::Instruction::IncC => disasm_inc8(soff, Reg8::C),
            cpu::Instruction::IncDE => disasm_inc16(soff, Reg16::DE),
            cpu::Instruction::LoadAPtrDE => disasm_load_reg8_reg16ptr(soff, Reg8::A, Reg16::DE),
            cpu::Instruction::LoadAE => disasm_load_reg8_reg8(soff, Reg8::A, Reg8::E),
            cpu::Instruction::LoadAImm8 => disasm_load_reg8_imm8(soff, Reg8::A),
            cpu::Instruction::LoadBImm8 => disasm_load_reg8_imm8(soff, Reg8::B),
            cpu::Instruction::LoadCImm8 => disasm_load_reg8_imm8(soff, Reg8::C),
            cpu::Instruction::LoadDImm8 => disasm_load_reg8_imm8(soff, Reg8::D),
            cpu::Instruction::LoadEImm8 => disasm_load_reg8_imm8(soff, Reg8::E),
            cpu::Instruction::LoadHImm8 => disasm_load_reg8_imm8(soff, Reg8::H),
            cpu::Instruction::LoadLImm8 => disasm_load_reg8_imm8(soff, Reg8::L),
            cpu::Instruction::LoadBCImm16 => disasm_load_reg16_imm16(soff, Reg16::BC),
            cpu::Instruction::LoadDEImm16 => disasm_load_reg16_imm16(soff, Reg16::DE),
            cpu::Instruction::LoadHLImm16 => disasm_load_reg16_imm16(soff, Reg16::HL),
            cpu::Instruction::LoadSPImm16 => disasm_load_reg16_imm16(soff, Reg16::SP),
            cpu::Instruction::StoreAToAddrCHigh => {
                disasm_store_reg8ptr_reg8(soff, Reg8::A, Reg8::C)
            }
            cpu::Instruction::StoreAToAddrImm8High => disasm_store_imm8ptr_reg8(soff, Reg8::A),
            cpu::Instruction::StoreAToAddrHL => {
                disasm_store_reg16ptr_reg8(soff, Reg8::A, Reg16::HL)
            }
            cpu::Instruction::StoreAToAddrHLDec => {
                disasm_store_reg16ptr_reg8_dec(soff, Reg8::A, Reg16::HL)
            }
            cpu::Instruction::XorA8 => disassemble_xor8(soff, Reg8::A),
            _ => panic!("disassemble: unexpected instruction {:?}", instruction),
        };
        offset += n;
        println!("{}", &s);
    }
}

fn disasm_call(stream: &[u8]) -> (String, usize) {
    return (format!("CALL {}", Imm16(stream[1], stream[2])), 3);
}

fn disasm_cb_prefixed(stream: &[u8]) -> (String, usize) {
    let stream_at_offset = &stream[1..];

    let instruction = cpu::Instruction::decode(stream_at_offset[0]);
    let (s, n) = match instruction {
        cpu::Instruction::BitTest7H => disassemble_test_bit_8(stream_at_offset, Reg8::H, 7),
        _ => panic!(":("),
    };
    (s, n + 1)
}

fn disassemble_test_bit_8(_stream: &[u8], reg: Reg8, bit: i64) -> (String, usize) {
    return (format!("BIT {},{}", bit, reg), 1);
}
// fn disassemble_dec_8(stream: &[u8], reg: Reg8) -> (String, usize) {
//     return (format!("INC {}", reg), 1);
// }

// fn disassemble_dec_16(stream: &[u8], reg: Reg16) -> (String, usize) {
//     return (format!("DEC {}", reg), 1);
// }

fn disasm_inc8(_stream: &[u8], reg: Reg8) -> (String, usize) {
    return (format!("INC {}", reg), 1);
}

fn disasm_inc16(_stream: &[u8], reg: Reg16) -> (String, usize) {
    return (format!("INC {}", reg), 1);
}

fn disasm_jmpr(stream: &[u8], condition: JmpCondition) -> (String, usize) {
    return (format!("JR {},{}", condition, Imm8(stream[1])), 2);
}

fn disasm_load_reg8_reg16ptr(_stream: &[u8], reg_dest: Reg8, reg_addr: Reg16) -> (String, usize) {
    return (format!("LD {},({})", reg_dest, reg_addr), 1);
}

fn disasm_load_reg8_reg8(stream: &[u8], reg_dest: Reg8, reg_src: Reg8) -> (String, usize) {
    return (format!("LD {},{}", reg_dest, reg_src), 1);
}

fn disasm_load_reg8_imm8(stream: &[u8], reg: Reg8) -> (String, usize) {
    return (format!("LD {},{}", reg, Imm8(stream[1])), 2);
}

fn disasm_load_reg16_imm16(stream: &[u8], reg: Reg16) -> (String, usize) {
    return (format!("LD {},{}", reg, Imm16(stream[1], stream[2])), 3);
}

fn disasm_store_imm8ptr_reg8(stream: &[u8], reg8_value: Reg8) -> (String, usize) {
    return (format!("LD ($FF00+{}),{}", Imm8(stream[1]), reg8_value), 2);
}

fn disasm_store_reg8ptr_reg8(_stream: &[u8], reg8_value: Reg8, reg8_addr: Reg8) -> (String, usize) {
    return (format!("LD ($FF00+{}),{}", reg8_addr, reg8_value), 1);
}

fn disasm_store_reg16ptr_reg8(_stream: &[u8], reg8: Reg8, reg16: Reg16) -> (String, usize) {
    return (format!("LD ({}),{}", reg16, reg8), 1);
}

fn disasm_store_reg16ptr_reg8_dec(_stream: &[u8], reg8: Reg8, reg16: Reg16) -> (String, usize) {
    return (format!("LD ({}-),{}", reg16, reg8), 1);
}

fn disassemble_xor8(_stream: &[u8], reg: Reg8) -> (String, usize) {
    return (format!("XOR {}", reg), 1);
}
