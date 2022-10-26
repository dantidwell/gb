use crate::bus;

#[derive(Debug)]
enum Reg8 {
    None,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Imm8,
    SPLow,
    SPHigh,
}

#[derive(Debug)]
enum Reg16 {
    None,
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug)]
pub enum Instruction {
    BitTest7H,
    Call,
    CBPrefix,
    JmpRelIfNonZero,
    IncC,
    IncDE,
    LoadAPtrDE,
    LoadAE,
    LoadAImm8,
    LoadBImm8,
    LoadCImm8,
    LoadDImm8,
    LoadEImm8,
    LoadHImm8,
    LoadLImm8,
    LoadBCImm16,
    LoadDEImm16,
    LoadHLImm16,
    LoadSPImm16,
    StoreAToAddrImm8High,
    StoreAToAddrCHigh,
    StoreAToAddrHL,
    StoreAToAddrHLDec,
    XorA8,
}

impl Instruction {
    pub fn decode(opcode: u8) -> Self {
        match opcode {
            0x7C => Instruction::BitTest7H,
            0xCD => Instruction::Call,
            0xCB => Instruction::CBPrefix,
            0x20 => Instruction::JmpRelIfNonZero,
            0x0C => Instruction::IncC,
            0x13 => Instruction::IncDE,
            0x1A => Instruction::LoadAPtrDE,
            0x7B => Instruction::LoadAE,
            0x3E => Instruction::LoadAImm8,
            0x06 => Instruction::LoadBImm8,
            0x0E => Instruction::LoadCImm8,
            0x16 => Instruction::LoadDImm8,
            0x1E => Instruction::LoadEImm8,
            0x26 => Instruction::LoadHImm8,
            0x2E => Instruction::LoadLImm8,
            0x01 => Instruction::LoadBCImm16,
            0x11 => Instruction::LoadDEImm16,
            0x21 => Instruction::LoadHLImm16,
            0x31 => Instruction::LoadSPImm16,
            0xE0 => Instruction::StoreAToAddrImm8High,
            0xE2 => Instruction::StoreAToAddrCHigh,
            0x77 => Instruction::StoreAToAddrHL,
            0x32 => Instruction::StoreAToAddrHLDec,
            0xAF => Instruction::XorA8,
            _ => panic!("deocde: invalid opcode value {:#x}", opcode),
        }
    }
}

const FLAG_ZERO: u8 = 1 << 7;
const FLAG_SUB: u8 = 1 << 6;
const FLAG_H_CARRY: u8 = 1 << 5;
const FLAG_CARRY: u8 = 1 << 4;

enum BusOp {
    Read,
    Write,
}

struct BusRequest {
    address: u16,
    op: BusOp,
    read_dest: Reg8,
    write_value: u8,
}

impl BusRequest {
    fn read(address: u16, read_dest: Reg8) -> Self {
        Self {
            op: BusOp::Read,
            address,
            read_dest,
            write_value: 0,
        }
    }

    fn write(address: u16, write_value: u8) -> Self {
        Self {
            op: BusOp::Write,
            address,
            write_value,
            read_dest: Reg8::None,
        }
    }
}

enum ExecOp {
    Nop,
    Dec16,
    Inc8,
    JmpRelative,
}

struct ExecRequest {
    op: ExecOp,
    op8_1: Reg8,
    op8_2: Reg8,
    op16_1: Reg16,
    op16_2: Reg16,
}

impl ExecRequest {
    fn nop() -> Self {
        Self {
            op: ExecOp::Nop,
            op8_1: Reg8::None,
            op8_2: Reg8::None,
            op16_1: Reg16::None,
            op16_2: Reg16::None,
        }
    }

    fn op8(op: ExecOp, arg: Reg8) -> Self {
        Self {
            op,
            op8_1: arg,
            op8_2: Reg8::None,
            op16_1: Reg16::None,
            op16_2: Reg16::None,
        }
    }

    fn op16(op: ExecOp, arg: Reg16) -> Self {
        Self {
            op,
            op8_1: Reg8::None,
            op8_2: Reg8::None,
            op16_1: arg,
            op16_2: Reg16::None,
        }
    }
}

pub struct Cpu<'a> {
    a: i8,
    b: i8,
    c: i8,
    d: i8,
    e: i8,
    h: i8,
    l: i8,
    imm8: i8,

    flags: u8,
    sp: u16,
    pc: u16,
    ticks: u64,

    bus_queue: std::collections::VecDeque<BusRequest>,
    exec_queue: std::collections::VecDeque<ExecRequest>,

    bus: &'a mut bus::Bus<'a>,
}

impl<'a> Cpu<'a> {
    pub fn new(bus: &'a mut bus::Bus<'a>) -> Self {
        return Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            flags: 0,
            h: 0,
            l: 0,
            imm8: 0,
            sp: 0,
            pc: 0,
            ticks: 0,
            bus_queue: std::collections::VecDeque::new(),
            exec_queue: std::collections::VecDeque::new(),

            bus,
        };
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
        match self.bus_queue.pop_front() {
            None => self.do_decode(),
            Some(req) => self.do_bus_read_write(req),
        }
        match self.exec_queue.pop_front() {
            None => (),
            Some(req) => self.do_execute(req),
        }
        self.pc = self.pc + 1;
    }

    fn do_bus_read_write(&mut self, req: BusRequest) {
        match req.op {
            BusOp::Read => self.set_reg8(req.read_dest, self.bus.read(req.address) as i8),
            BusOp::Write => self.bus.write(req.address, req.write_value),
        }
    }

    fn do_decode(&mut self) {
        let opcode = Instruction::decode(self.bus.read(self.pc));

        match opcode {
            Instruction::BitTest7H => self.exec_bit_test_8(Reg8::H, 7),
            Instruction::CBPrefix => self.cb(),
            Instruction::JmpRelIfNonZero => self.queue_jmp_relative((!self.flags & FLAG_ZERO) > 0),
            Instruction::IncC => self.queue_inc_8(Reg8::C),
            Instruction::LoadAImm8 => self.queue_load_imm_8(Reg8::A),
            Instruction::LoadCImm8 => self.queue_load_imm_8(Reg8::C),
            Instruction::LoadHLImm16 => self.queue_load_imm_16(Reg8::L, Reg8::H),
            Instruction::LoadSPImm16 => self.queue_load_imm_16(Reg8::SPLow, Reg8::SPHigh),
            Instruction::StoreAToAddrHL => self.queue_store_to_addr_in_hl(self.a),
            Instruction::StoreAToAddrHLDec => self.queue_store_a_to_addr_in_hl_dec(self.a),
            Instruction::StoreAToAddrCHigh => self.queue_store_a_to_addr_in_c(),
            Instruction::XorA8 => self.exec_xor_8(Reg8::A),
            _ => panic!("cpu: instruction not implemented {:?}", opcode),
        }
    }

    fn do_execute(&mut self, req: ExecRequest) {
        match req.op {
            ExecOp::Dec16 => self.exec_dec_16(req.op16_1),
            ExecOp::Inc8 => self.exec_inc_8(req.op8_1),
            ExecOp::JmpRelative => self.exec_jmp_rel(),
            ExecOp::Nop => (),
            // _ => panic!("do_execute: invalid operation"),
        }
    }

    fn exec_bit_test_8(&mut self, reg: Reg8, bit: i64) {
        let res: i8 = match reg {
            Reg8::A => self.a & (1 << bit),
            Reg8::B => self.b & (1 << bit),
            Reg8::C => self.c & (1 << bit),
            Reg8::D => self.d & (1 << bit),
            Reg8::E => self.e & (1 << bit),
            Reg8::H => self.h & (1 << bit),
            Reg8::L => self.l & (1 << bit),
            _ => panic!("bit_test_8: invalid operand"),
        };
        self.flags = FLAG_H_CARRY | (self.flags & FLAG_CARRY);
        if res == 0 {
            self.flags |= FLAG_ZERO;
        }
    }

    fn exec_dec_16(&mut self, reg: Reg16) {
        match reg {
            Reg16::HL => {
                let mut hl = ((self.h as i16) << 8) | (self.l as i16 & 0x00FF);
                hl = hl.wrapping_sub(1);

                self.flags = 0;
                if hl == 0 {
                    self.flags |= FLAG_ZERO;
                }
                self.h = (hl >> 8) as i8;
                self.l = hl as i8;
            }
            _ => panic!("exec_dec_16: invalid operand"),
        }
    }

    fn exec_inc_8(&mut self, reg: Reg8) {
        match reg {
            Reg8::C => {
                self.c += 1;
            }
            _ => panic!("exec_inc_u: invalid operand"),
        }
    }

    fn exec_jmp_rel(&mut self) {
        let signed_pc = self.pc as i16;
        let signed_imm8 = self.imm8 as i16;
        let signed_result = signed_pc + signed_imm8;
        self.pc = signed_result as u16;
    }

    fn exec_xor_8(&mut self, reg: Reg8) {
        match reg {
            Reg8::A => self.a ^= self.a,
            Reg8::B => self.a ^= self.b,
            Reg8::C => self.a ^= self.c,
            Reg8::D => self.a ^= self.d,
            Reg8::E => self.a ^= self.e,
            Reg8::H => self.a ^= self.h,
            Reg8::L => self.a ^= self.l,
            _ => panic!("xor_8: invalid operand"),
        }
        self.flags = 0;
        if self.a == 0 {
            self.flags |= FLAG_ZERO;
        }
    }

    fn queue_bus_read(&mut self, addr: u16, read_dest: Reg8) {
        self.bus_queue.push_back(BusRequest::read(addr, read_dest));
    }

    fn queue_bus_write(&mut self, addr: u16, write_value: i8) {
        self.bus_queue
            .push_back(BusRequest::write(addr, write_value as u8))
    }

    fn queue_inc_8(&mut self, reg: Reg8) {
        self.exec_queue
            .push_back(ExecRequest::op8(ExecOp::Inc8, reg))
    }

    fn queue_jmp_relative(&mut self, condition: bool) {
        self.queue_bus_read(self.pc + 1, Reg8::Imm8);
        if condition {
            self.exec_queue.push_back(ExecRequest::nop());
            self.exec_queue
                .push_back(ExecRequest::op8(ExecOp::JmpRelative, Reg8::Imm8));
        }
    }

    fn queue_load_imm_8(&mut self, reg: Reg8) {
        self.queue_bus_read(self.pc + 1, reg);
    }

    fn queue_load_imm_16(&mut self, reg_low: Reg8, reg_high: Reg8) {
        self.queue_bus_read(self.pc + 1, reg_low);
        self.queue_bus_read(self.pc + 2, reg_high);
    }

    fn queue_store_a_to_addr_in_c(&mut self) {
        let addr = 0xFF00 | (self.c as u16 & 0x00FF);
        self.queue_bus_write(addr, self.a);
    }

    fn queue_store_to_addr_in_hl(&mut self, value: i8) {
        let hl: u16 = (self.h as u16) << 8 | (self.l as u16 & 0x00FF);
        self.queue_bus_write(hl, value)
    }

    fn queue_store_a_to_addr_in_hl_dec(&mut self, value: i8) {
        let hl: u16 = (self.h as u16) << 8 | (self.l as u16 & 0x00FF);

        self.queue_bus_write(hl, value);
        self.exec_queue.push_back(ExecRequest::nop());
        self.exec_queue
            .push_back(ExecRequest::op16(ExecOp::Dec16, Reg16::HL));
    }

    fn cb(&mut self) {}

    fn set_reg8(&mut self, reg: Reg8, val: i8) {
        match reg {
            Reg8::A => self.a = val,
            Reg8::B => self.b = val,
            Reg8::C => self.c = val,
            Reg8::D => self.d = val,
            Reg8::E => self.e = val,
            Reg8::H => self.h = val,
            Reg8::L => self.l = val,
            Reg8::Imm8 => self.imm8 = val,
            Reg8::SPLow => self.sp = (self.sp & 0xFF00) | (val as u16 & 0x00FF),
            Reg8::SPHigh => self.sp = (self.sp & 0x00FF) | ((val as u16 & 0x00FF) << 8),
            _ => panic!("invalid register"),
        }
    }
}
