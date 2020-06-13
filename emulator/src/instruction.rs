pub type Register = usize;
pub type Address = u16;
pub type Value = u8;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Invalid,
    Sys(Address),
    Cls,
    Ret,
    Jump(Address),
    Call(Address),
    SkipValEq(Register, Value),
    SkipValNotEq(Register, Value),
    SkipEq(Register, Register),
    LoadVal(Register, Value),
    AddVal(Register, Value),
    Load(Register, Register),
    Or(Register, Register),
    And(Register, Register),
    Xor(Register, Register),
    Add(Register, Register),
    Sub(Register, Register),
    ShiftRight(Register),
    SubN(Register, Register),
    ShiftLeft(Register),
    SkipNotEq(Register, Register),
    LoadAddr(Address),
    JumpRel(Address),
    Random(Register, Value),
    Draw(Register, Register, Value),
    SkipKeyPressed(Register),
    SkipKeyNotPressed(Register),
    LoadDelayTimer(Register),
    LoadKey(Register),
    SetDelayTimer(Register),
    SetSoundTimer(Register),
    AddI(Register),
    LoadSprite(Register),
    Bcd(Register),
    StoreRegs(Register),
    LoadRegs(Register),
}

fn instr_ptr(pfx: u8, ptr: Address) -> [u8; 2] {
    assert!(pfx <= 0x0F);
    assert!(ptr <= 0x0FFF);

    [ (pfx << 4) | ((ptr & 0x0F00) >> 8) as u8,
      (ptr & 0x00FF) as u8 ]
}

fn instr_val(pfx: u8, reg: Register, val: Value) -> [u8; 2] {
    assert!(pfx <= 0x0F);
    assert!(reg <= 0x0F);

    [ (pfx << 4 | (reg as u8)), val ]
}

fn instr_reg(pfx: u8, dst: Register, src: Register, sfx: Value) -> [u8; 2] {
    assert!(pfx <= 0x0F);
    assert!(dst <= 0x0F);
    assert!(src <= 0x0F);
    assert!(sfx <= 0x0F);

    [ (pfx << 4 | (dst as u8)), ((src as u8) << 4 | sfx)]
}

macro_rules! reg1 {
    ( $x:expr ) => { (($x & 0x0F00) >> 8) as Register }
}

macro_rules! reg2 {
    ( $x:expr ) => { (($x & 0x00F0) >> 4) as Register }
}

macro_rules! val {
    ( $x:expr ) => { ($x & 0x00FF) as Value }
}
macro_rules! addr {
    ( $x:expr ) => { ($x & 0x0FFF) as Address }
}
macro_rules! nibble {
    ( $x:expr ) => { ($x & 0x000F) as Value }
}

impl Instruction {
    pub fn asm(&self) -> [u8; 2] {
        match self {
            Instruction::Invalid => [ 0x00, 0x00 ],
            Instruction::Sys(addr) => instr_ptr(0x0, *addr),
            Instruction::Cls => [ 0x00, 0xE0 ],                                     // Tested
            Instruction::Ret => [ 0x00, 0xEE ],                                     // Tested
            Instruction::Jump(addr) => instr_ptr(0x1, *addr),                       // Tested
            Instruction::Call(addr) => instr_ptr(0x2, *addr),                       // Tested
            Instruction::SkipValEq(reg, val) => instr_val(0x3, *reg, *val),         // Tested
            Instruction::SkipValNotEq(reg, val) => instr_val(0x4, *reg, *val),      // Tested
            Instruction::SkipEq(dst, src) => instr_reg(0x5, *dst, *src, 0),         // Tested
            Instruction::LoadVal(reg, val) => instr_val(0x6, *reg, *val),           // Tested
            Instruction::AddVal(reg, val) => instr_val(0x7, *reg, *val),            // Tested
            Instruction::Load(reg1, reg2) => instr_reg(0x8, *reg1, *reg2, 0x00),    // Tested
            Instruction::Or(reg1, reg2) => instr_reg(0x8, *reg1, *reg2, 0x1),       // Tested
            Instruction::And(reg1, reg2) => instr_reg(0x8, *reg1, *reg2, 0x2),      // Tested
            Instruction::Xor(reg1, reg2) => instr_reg(0x8, *reg1, *reg2, 0x3),      // Tested
            Instruction::Add(reg1, reg2) => instr_reg(0x8, *reg1, *reg2, 0x4),      // Tested
            Instruction::Sub(reg1, reg2) => instr_reg(0x8, *reg1, *reg2, 0x5),      // Tested
            Instruction::ShiftRight(reg) => instr_reg(0x8, *reg, 0, 0x6),           // Tested
            Instruction::SubN(reg1, reg2) => instr_reg(0x8, *reg1, *reg2, 0x7),     // Tested
            Instruction::ShiftLeft(reg) => instr_reg(0x8, *reg, 0, 0xE),            // Tested
            Instruction::SkipNotEq(reg1, reg2) => instr_reg(0x9, *reg1, *reg2, 0),  // Tested
            Instruction::LoadAddr(addr) => instr_ptr(0xA, *addr),                   // Tested
            Instruction::JumpRel(addr) => instr_ptr(0xB, *addr),
            Instruction::Random(reg, val) => instr_val(0xC, *reg, *val),
            Instruction::Draw(x, y, n) => instr_reg(0xD, *x, *y, *n),               // Tested
            Instruction::SkipKeyPressed(reg) => instr_reg(0xE, *reg, 0x9, 0xE),
            Instruction::SkipKeyNotPressed(reg) => instr_reg(0xE, *reg, 0xA, 0x1),
            Instruction::LoadDelayTimer(reg) => instr_reg(0xF, *reg, 0x0, 0x7),
            Instruction::LoadKey(reg) => instr_reg(0xF, *reg, 0x0, 0xA),
            Instruction::SetDelayTimer(reg) => instr_reg(0xF, *reg, 0x1, 0x5),
            Instruction::SetSoundTimer(reg) => instr_reg(0xF, *reg, 0x1, 0x8),
            Instruction::AddI(reg) => instr_val(0xF, *reg, 0x1E),                   // Tested
            Instruction::LoadSprite(reg) => instr_val(0xF, *reg, 0x29),             // Tested
            Instruction::Bcd(reg) => instr_val(0xF, *reg, 0x33),                    // Tested
            Instruction::StoreRegs(reg) => instr_val(0xF, *reg, 0x55),              // Tested
            Instruction::LoadRegs(reg) => instr_val(0xF, *reg, 0x65),               // Tested
        }
    }

    pub fn from(instr: u16) -> Instruction {
        match instr {
            0x00E0 => Instruction::Cls,
            0x00EE => Instruction::Ret,
            0x0000..=0x0FFF if addr!(instr) != 0x0E0 && addr!(instr) != 0x0EE => Instruction::Sys(addr!(instr)),
            0x1000..=0x1FFF => Instruction::Jump(addr!(instr)),
            0x2000..=0x2FFF => Instruction::Call(addr!(instr)),
            0x3000..=0x3FFF => Instruction::SkipValEq(reg1!(instr), val!(instr)),
            0x4000..=0x4FFF => Instruction::SkipValNotEq(reg1!(instr), val!(instr)),
            0x5000..=0x5FFF if instr & 0x000F == 0 => Instruction::SkipEq(reg1!(instr), reg2!(instr)),
            0x6000..=0x6FFF => Instruction::LoadVal(reg1!(instr), val!(instr)),
            0x7000..=0x7FFF => Instruction::AddVal(reg1!(instr), val!(instr)),
            0x8000..=0x8FFF => match instr & 0x000F {
                0x0 => Instruction::Load(reg1!(instr), reg2!(instr)),
                0x1 => Instruction::Or(reg1!(instr), reg2!(instr)),
                0x2 => Instruction::And(reg1!(instr), reg2!(instr)),
                0x3 => Instruction::Xor(reg1!(instr), reg2!(instr)),
                0x4 => Instruction::Add(reg1!(instr), reg2!(instr)),
                0x5 => Instruction::Sub(reg1!(instr), reg2!(instr)),
                0x6 => Instruction::ShiftRight(reg1!(instr)),
                0x7 => Instruction::SubN(reg1!(instr), reg2!(instr)),
                0xE => Instruction::ShiftLeft(reg1!(instr)),
                _ => panic!("Invalid 0x8... instruction"),
            },
            0x9000..=0x9FFF if instr & 0x000F == 0 => {
                Instruction::SkipNotEq(reg1!(instr), reg2!(instr))
            },
            0xA000..=0xAFFF => Instruction::LoadAddr(addr!(instr)),
            0xB000..=0xBFFF => Instruction::JumpRel(addr!(instr)),
            0xC000..=0xCFFF => Instruction::Random(reg1!(instr), val!(instr)),
            0xD000..=0xDFFF => Instruction::Draw(reg1!(instr), reg2!(instr), nibble!(instr)),
            0xE000..=0xEFFF if instr & 0xFF == 0x9E => Instruction::SkipKeyPressed(reg1!(instr)),
            0xE000..=0xEFFF if instr & 0xFF == 0xA1 => Instruction::SkipKeyNotPressed(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x07 => Instruction::LoadDelayTimer(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x0A => Instruction::LoadKey(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x15 => Instruction::SetDelayTimer(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x18 => Instruction::SetSoundTimer(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x1E => Instruction::AddI(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x29 => Instruction::LoadSprite(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x33 => Instruction::Bcd(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x55 => Instruction::StoreRegs(reg1!(instr)),
            0xF000..=0xFFFF if instr & 0xFF == 0x65 => Instruction::LoadRegs(reg1!(instr)),
            _ => panic!("Invalid instruction {:x}", instr)
        }
    }
}


#[cfg(test)]
mod test_instruction {
    use super::*;

    #[test]
    fn test_instructions() {
        let tests: Vec<(Instruction, [u8; 2])> = vec![
            (Instruction::Cls, [0x00, 0xE0]),
            (Instruction::Ret, [0x00, 0xEE]),
            (Instruction::Sys(0x123), [0x01, 0x23]),
            (Instruction::Jump(0x0234), [0x12, 0x34]),
            (Instruction::Call(0x0345), [0x23, 0x45]),
            (Instruction::SkipValEq(0x4, 0x56), [0x34, 0x56]),
            (Instruction::SkipValNotEq(0x5, 0x67), [0x45, 0x67]),
            (Instruction::SkipEq(0x6, 0x7), [0x56, 0x70]),
            (Instruction::LoadVal(0x7, 0x89), [0x67, 0x89]),
            (Instruction::AddVal(0x8, 0x90), [0x78, 0x90]),
            (Instruction::Load(1, 2), [0x81, 0x20]),
            (Instruction::Or(1, 2), [0x81, 0x21]),
            (Instruction::And(1, 2), [0x81, 0x22]),
            (Instruction::Xor(1, 2), [0x81, 0x23]),
            (Instruction::Add(1, 2), [0x81, 0x24]),
            (Instruction::Sub(1, 2), [0x81, 0x25]),
            (Instruction::ShiftRight(1), [0x81, 0x06]),
            (Instruction::SubN(1, 2), [0x81, 0x27]),
            (Instruction::ShiftLeft(1), [0x81, 0x0E]),
            (Instruction::SkipNotEq(0xA, 0xB), [0x9A, 0xB0]),
            (Instruction::LoadAddr(0xBCD), [0xAB, 0xCD]),
            (Instruction::JumpRel(0xCDE), [0xBC, 0xDE]),
            (Instruction::Random(0xD, 0xEF), [0xCD, 0xEF]),
            (Instruction::Draw(1, 2, 3), [0xD1, 0x23]),
            (Instruction::SkipKeyPressed(0xF), [0xEF, 0x9E]),
            (Instruction::SkipKeyNotPressed(0xF), [0xEF, 0xA1]),
            (Instruction::LoadDelayTimer(0), [0xF0, 0x07]),
            (Instruction::LoadKey(0), [0xF0, 0x0A]),
            (Instruction::SetDelayTimer(0), [0xF0, 0x15]),
            (Instruction::SetSoundTimer(0), [0xF0, 0x18]),
            (Instruction::AddI(0), [0xF0, 0x1E]),
            (Instruction::LoadSprite(1), [0xF1, 0x29]),
            (Instruction::Bcd(1), [0xF1, 0x33]),
            (Instruction::StoreRegs(1), [0xF1, 0x55]),
            (Instruction::LoadRegs(1), [0xF1, 0x65]),
        ];

        for (instr, ops) in tests {
            assert_eq!(instr.asm(), ops);
            assert_eq!(Instruction::from(((ops[0] as u16) << 8) | (ops[1] as u16)), instr);
        }
    }
}
