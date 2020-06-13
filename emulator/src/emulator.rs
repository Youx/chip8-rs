pub mod instruction;
pub mod ui;

extern crate rand;

use crate::instruction::{Instruction, Register, Value, Address};
use std::time::{Duration, SystemTime};

pub enum DisplaySize {
    Basic64x32,
    Eti64x48,
    Eti64x64,
    Hp128x64,
}

const SPRITES : [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
];

// 60Hz timers = 16ms period
const TICK :Duration = Duration::from_nanos(1000 * 1000 * 1000 / 60);

pub struct Emulator {
  memory: [u8; 4096],
  /* internal registers */
  pc_reg: u16,
  sp_reg: u8,

  /* keyboard state */
  pub keys: [bool; 16],

  /* general purpose */
  regs: [u8; 16],

  /* ptr register */
  i_reg: u16,

  /* delay timer and sound timer */
  dt_reg: u8,
  st_reg: u8,
  last_tick: SystemTime,
  frequency: u64, // Hz

  /* return addr stack */
  stack: [u16; 16],

  /* screen, size may vary depending on configuration */
  pub resolution: (usize, usize),
  pub screen: Vec<Vec<bool>>,
  pub redraw: bool,
}

impl Emulator {
    pub fn new_with_time(display: DisplaySize, now: SystemTime) -> Self where Self: Sized {
        let resolution = Emulator::get_resolution(display);

        let mut emu = Emulator {
            pc_reg: 0x200,
            sp_reg: 0,
            i_reg: 0,

            dt_reg: 0,
            st_reg: 0,
            last_tick: now,
            frequency: 6000,

            memory: [0; 4096],
            regs: [0; 16],
            stack: [0; 16],
            resolution,
            screen: vec![vec![false; resolution.1]; resolution.0],
            redraw: false,
            keys: [false; 16],
        };
        emu.init_sprites();
        emu
    }
    pub fn new(display: DisplaySize) -> Self where Self: Sized {
        Emulator::new_with_time(display, SystemTime::now())
    }
    fn get_resolution(display: DisplaySize) -> (usize, usize) {
        match display {
            DisplaySize::Basic64x32 => { (64, 32) }
            DisplaySize::Eti64x48 => { (64, 48) }
            DisplaySize::Eti64x64 => { (64, 64) }
            DisplaySize::Hp128x64 => { (128, 64) }
        }
    }
    fn set_screen_mode(&mut self, resolution: DisplaySize) {
        let resolution = Emulator::get_resolution(resolution);
        self.resolution = resolution;
        self.screen = vec![vec![false; resolution.1]; resolution.0]
    }
    fn init_sprites(&mut self) {
        for (idx, x) in SPRITES.iter().enumerate() {
            self.memory[idx] = *x;
        }
    }
    pub fn reset(&mut self) {
        self.pc_reg = 0x200;
        self.sp_reg = 0;
        self.regs[0xF] = 0;
        self.screen.iter_mut().for_each(|x| {
            x.iter_mut().for_each(|y| *y = false);
        });
        self.memory.iter_mut().for_each(|x| *x = 0);
        self.init_sprites();
        self.regs.iter_mut().for_each(|x| *x = 0);
        self.i_reg = 0;
        self.dt_reg = 0;
        self.st_reg = 0;
        self.stack.iter_mut().for_each(|x| *x = 0);
        self.keys.iter_mut().for_each(|x| *x = false);
    }
    pub fn mem_load_bin(&mut self, data: Vec<u8>) {
        for (idx, x) in data.iter().enumerate() {
            self.memory[0x200 + idx ] = *x;
        }
    }
    #[cfg(test)]
    fn mem_load_instr(&mut self, data: Vec<Instruction>) {
        let bytes : Vec<u8> = data.iter().map(|instruction|
            instruction.asm()
        ).collect::<Vec<[u8;2]>>().concat();
        self.mem_load_bin(bytes);
    }

    fn tick(&mut self) {
        let mut elapsed = self.last_tick.elapsed().unwrap();

        if elapsed > TICK {
            while elapsed > TICK {
                if self.st_reg > 0 {
                    self.st_reg -= 1;
                }
                if self.dt_reg > 0 {
                    self.dt_reg -= 1;
                }
                elapsed -= TICK;
            }
            self.last_tick = SystemTime::now();
        }
    }
    fn tick_with_time(&mut self, now: SystemTime) {
        let mut elapsed = now.duration_since(self.last_tick).unwrap();
        if elapsed > TICK {
            while elapsed > TICK {
                if self.st_reg > 0 {
                    self.st_reg -= 1;
                }
                if self.dt_reg > 0 {
                    self.dt_reg -= 1;
                }
                elapsed -= TICK;
            }
            self.last_tick = now;
        }
    } 
    pub fn cpu_one_cycle_with_time(&mut self, now: SystemTime) {
        let instr = self.cpu_load();

        self.tick_with_time(now);
        self.cpu_exec(instr);
    }
    pub fn cpu_one_cycle(&mut self) {
        let instr = self.cpu_load();

        self.tick();
        self.cpu_exec(instr);
    }
    fn cpu_load(&mut self) -> u16 {
        let instr : u16 = ( (self.memory[self.pc_reg as usize] as u16) << 8
                          | (self.memory[self.pc_reg as usize + 1] as u16)).into();

        self.inc_pc();
        instr
    }
    fn cpu_exec(&mut self, instr: u16) {
        match Instruction::from(instr) {
            /* 2 special cases */
            Instruction::Sys(0x230) => self.cls(), // TODO: test
            Instruction::Jump(0x1260) if self.pc_reg == 0x202 => self.hires(), // TODO: test

            Instruction::Cls => self.cls(),
            Instruction::Ret => self.ret(),
            Instruction::Sys(_) => {},
            Instruction::Jump(addr) => self.jump(addr),
            Instruction::Call(addr) => self.call(addr),
            Instruction::SkipValEq(reg, val) => self.skip_val_equal(reg, val),
            Instruction::SkipValNotEq(reg, val) => self.skip_val_notequal(reg, val),
            Instruction::SkipEq(reg1, reg2) => self.skip_reg_equal(reg1, reg2),
            Instruction::LoadVal(reg, val) => self.load_val(reg, val),
            Instruction::AddVal(reg, val) => self.add_val(reg, val),
            Instruction::Load(dst, src) => self.load(dst, src),
            Instruction::Or(dst, src) => self.or(dst, src),
            Instruction::And(dst, src) => self.and(dst, src),
            Instruction::Xor(dst, src) => self.xor(dst, src),
            Instruction::Add(dst, src) => self.add(dst, src),
            Instruction::Sub(dst, src) => self.sub(dst, src),
            Instruction::ShiftRight(reg) => self.shr(reg),
            Instruction::SubN(dst, src) => self.subn(dst, src),
            Instruction::ShiftLeft(reg) => self.shl(reg),
            Instruction::SkipNotEq(reg1, reg2) => self.skip_reg_not_equal(reg1, reg2),
            Instruction::LoadAddr(addr) => self.load_addr(addr),
            Instruction::JumpRel(addr) => self.jump_v0(addr),
            Instruction::Random(reg, val) => self.rand(reg, val),
            Instruction::Draw(xreg, yreg, nibble) => self.draw(xreg, yreg, nibble),
            Instruction::SkipKeyPressed(reg) => self.skp(reg),
            Instruction::SkipKeyNotPressed(reg) => self.sknp(reg),
            Instruction::LoadDelayTimer(reg) => self.dt_to_vx(reg),
            Instruction::LoadKey(reg) => self.load_key(reg),
            Instruction::SetDelayTimer(reg) => self.vx_to_dt(reg),
            Instruction::SetSoundTimer(reg) => self.load_st(reg),
            Instruction::AddI(reg) => self.addi(reg),
            Instruction::LoadSprite(reg) => self.loadi_sprite(reg),
            Instruction::Bcd(reg) => self.bcd(reg),
            Instruction::StoreRegs(reg) => self.regs_to_mem(reg),
            Instruction::LoadRegs(reg) => self.mem_to_regs(reg),
            _ => panic!("Invalid instruction {:x}", instr)
        }
    }
    fn inc_pc(&mut self) {
        self.pc_reg += 2;
    }
    fn screen_draw(&mut self) {
        self.redraw = true;
    }

    fn hires(&mut self) {
        self.set_screen_mode(DisplaySize::Eti64x64);
        self.pc_reg = 0x2c0;
    }
    /// Clear display
    fn cls(&mut self) {
        for y in 0..self.resolution.1 {
            for x in 0..self.resolution.0 {
                self.screen[x][y] = false;
            }
        }
       self.screen_draw();
    }
    /// Return from subroutine
    fn ret(&mut self) {
        self.sp_reg -= 1;
        self.pc_reg = self.stack[self.sp_reg as usize];
    }
    /// Jump to address
    fn jump(&mut self, addr: Address) {
        self.pc_reg = addr;
    }
    /// Call function at address
    fn call(&mut self, addr: Address) {
        self.stack[self.sp_reg as usize] = self.pc_reg;
        self.sp_reg += 1;
        self.pc_reg = addr;
    }
    fn skip_val_equal(&mut self, reg: Register, val: Value) {
        if self.regs[reg] == val {
            self.inc_pc();
        }
    }
    fn skip_val_notequal(&mut self, reg: Register, val: Value) {
        if self.regs[reg] != val {
            self.inc_pc();
        }
    }
    fn skip_reg_equal(&mut self, reg1: Register, reg2: Register) {
        if self.regs[reg1] == self.regs[reg2] {
            self.inc_pc();
        }
    }
    fn load_val(&mut self, dst: Register, val: Value) {
        self.regs[dst] = val;
    }
    fn add_val(&mut self, dst: Register, val: Value) {
        self.regs[dst] = self.regs[dst].wrapping_add(val);
    }
    fn load(&mut self, dst: Register, src: Register) {
        self.regs[dst] = self.regs[src];
    }
    fn or(&mut self, dst: Register, src: Register) {
        self.regs[dst] |= self.regs[src];
    }
    fn and(&mut self, dst: Register, src: Register) {
        self.regs[dst] &= self.regs[src];
    }
    fn xor(&mut self, dst: Register, src: Register) {
        self.regs[dst] ^= self.regs[src];
    }
    fn add(&mut self, dst: Register, src: Register) {
        self.regs[0xF] = if self.regs[dst] as u16 + self.regs[src] as u16 > 255 { 1 } else { 0 };
        self.regs[dst] = self.regs[dst].wrapping_add(self.regs[src]);
    }
    fn sub(&mut self, dst: Register, src: Register) {
        self.regs[0xF] = if self.regs[dst] >= self.regs[src] { 1 } else { 0 };
        self.regs[dst] = self.regs[dst].wrapping_sub(self.regs[src]);
    }
    fn shr(&mut self, reg: Register) {
        self.regs[0xF] = if self.regs[reg] & 0x01 != 0 { 1 } else { 0 };
        self.regs[reg] >>= 1;
    }
    fn subn(&mut self, dst: Register, src: Register) {
        self.regs[0xF] = if self.regs[src] >= self.regs[dst] { 1 } else { 0 };
        self.regs[dst] = self.regs[src].wrapping_sub(self.regs[dst]);
    }
    fn shl(&mut self, reg: Register) {
        self.regs[0xF] = if self.regs[reg] & 128 != 0 { 1 } else { 0 };
        self.regs[reg] <<= 1;
    }
    fn skip_reg_not_equal(&mut self, dst: Register, src: Register) {
        if self.regs[dst] != self.regs[src] {
            self.inc_pc();
        }
    }
    fn load_addr(&mut self, addr: Address) {
        self.i_reg = addr;
    }
    fn jump_v0(&mut self, addr: Address) {
        self.pc_reg = self.regs[0] as u16 + addr;
    }
    fn rand(&mut self, reg: Register, val: Value) {
        self.regs[reg] = rand::random::<u8>() & val;
    }
    fn draw(&mut self, xreg: Register, yreg: Register, n: Value) {
        let x_start = self.regs[xreg] as usize;
        let y_start = self.regs[yreg] as usize;

        self.regs[0xF] = 0;
        for y in 0..n as usize {
            let line = self.memory[self.i_reg as usize + y];
            let screen_y = (y_start + y) % self.resolution.1;

            for x in 0..8 {
                let sprite_pixel : bool = line & (0x1 << 7 - x) != 0;
                let screen_x = (x_start + x) % self.resolution.0;
                let current_pixel = self.screen[screen_x][screen_y];

                // pixel will be erased, flag overflow
                if sprite_pixel && current_pixel {
                    self.regs[0xF] = 1;
                }
                self.screen[screen_x][screen_y] ^= sprite_pixel;
            }
        }
        self.screen_draw();
    }
    // skip if key pressed
    fn skp(&mut self, reg: Register) {
        if self.keys[self.regs[reg] as usize] {
            self.inc_pc();
        }
    }
    // skip if key not pressed
    fn sknp(&mut self, reg: Register) {
        if !self.keys[self.regs[reg] as usize] {
            self.inc_pc();
        }
    }
    fn dt_to_vx(&mut self, reg: Register) {
        self.regs[reg] = self.dt_reg;
    }
    fn load_key(&mut self, reg: Register) {
        let mut pressed = false;

        for (idx, k) in self.keys.iter().enumerate() {
            if *k {
                self.regs[reg] = idx as u8;
                pressed = true;
                break;
            }
        }
        // loop until a key is pressed
        if !pressed {
            self.pc_reg -= 2;
        }
    }
    fn vx_to_dt(&mut self, src: Register) {
        self.dt_reg = self.regs[src];
    }
    fn load_st(&mut self, src: Register) {
        self.st_reg = self.regs[src];
    }
    fn addi(&mut self, src: Register) {
        self.i_reg += self.regs[src] as u16;
    }
    fn loadi_sprite(&mut self, src: Register) {
        self.i_reg = self.regs[src] as u16 * 5;
    }
    fn bcd(&mut self, src: Register) {
        let mut val = self.regs[src];

        self.memory[self.i_reg as usize + 2] = val % 10;
        val /= 10;
        self.memory[self.i_reg as usize + 1] = val % 10;
        val /= 10;
        self.memory[self.i_reg as usize] = val;
    }
    fn regs_to_mem(&mut self, reg: Register) {
        for i in 0..=reg {
            self.memory[self.i_reg as usize + i] = self.regs[i];
        }
    }
    fn mem_to_regs(&mut self, reg: Register) {
        for i in 0..=reg {
            self.regs[i] = self.memory[self.i_reg as usize + i];
        }
    }
}

#[cfg(test)]
mod test_emu {
    use super::*;

    #[test]
    fn test_001_load_software() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_bin(vec![0x01, 0x02, 0x03, 0x04]);
        assert_eq!(emu.memory[0x200], 0x01);
        assert_eq!(emu.memory[0x201], 0x02);
        assert_eq!(emu.memory[0x202], 0x03);
        assert_eq!(emu.memory[0x203], 0x04);
    }

    #[test]
    fn test_010_loadval_addval() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_instr(vec![
            Instruction::LoadVal(1, 0x55),
            Instruction::AddVal(1, 0xAA),
            Instruction::AddVal(1, 0x01),
        ]);
        assert_eq!(emu.regs[1], 0x00);
        assert_eq!(emu.pc_reg, 0x200);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x202);
        assert_eq!(emu.regs[1], 0x55);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x204);
        assert_eq!(emu.regs[1], 0xFF);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x206);
        assert_eq!(emu.regs[1], 0x00);
    }

    #[test]
    fn test_011_load_add_sub() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_instr(vec![
            Instruction::LoadVal(1, 0x56), // $1 = 0x56
            Instruction::Load(2, 1),       // $2 = $1
            Instruction::Add(2, 2),        // $2 = $2 + $2 (0xAC)
            Instruction::Add(1, 2),        // $1 = $1 + $2 (0x02 + overflow)
            Instruction::Sub(1, 2),        // $1 = $1 - $2 (0x56 + no overflow)
            Instruction::Sub(1, 1),        // $1 = $1 - $1 (0x00 + overflow)
        ]);
        assert_eq!(emu.regs[1], 0x00);
        assert_eq!(emu.regs[2], 0x00);
        assert_eq!(emu.pc_reg, 0x200);

        emu.cpu_one_cycle();
        assert_eq!(emu.regs[1], 0x56);
        assert_eq!(emu.regs[2], 0x00);
        assert_eq!(emu.pc_reg, 0x202);

        emu.cpu_one_cycle();
        assert_eq!(emu.regs[1], 0x56);
        assert_eq!(emu.regs[2], 0x56);
        assert_eq!(emu.pc_reg, 0x204);

        emu.cpu_one_cycle();
        assert_eq!(emu.regs[1], 0x56);
        assert_eq!(emu.regs[2], 0xAC);
        assert_eq!(emu.regs[0xF], 0);
        assert_eq!(emu.pc_reg, 0x206);

        emu.cpu_one_cycle();
        assert_eq!(emu.regs[1], 0x02);
        assert_eq!(emu.regs[2], 0xAC);
        assert_eq!(emu.regs[0xF], 1);
        assert_eq!(emu.pc_reg, 0x208);

        emu.cpu_one_cycle();
        assert_eq!(emu.regs[1], 0x56);
        assert_eq!(emu.regs[2], 0xAC);
        assert_eq!(emu.regs[0xF], 0);
        assert_eq!(emu.pc_reg, 0x20A);

        emu.cpu_one_cycle();
        assert_eq!(emu.regs[1], 0x00);
        assert_eq!(emu.regs[0xF], 1);
        assert_eq!(emu.pc_reg, 0x20C);
    }

    #[test]
    fn test_012_or_and_xor_shr_shl() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_instr(vec![
            Instruction::LoadVal(1, 0xAA), // $1 = 0x55
            Instruction::LoadVal(2, 0x55), // $2 = 0x55
            Instruction::Or(1, 2),         // $1 = $1 | $2 (0xFF)

            Instruction::LoadVal(1, 0xAA), // $1 = 0x55
            Instruction::And(1, 2),        // $1 = $1 & $2 (0x00)

            Instruction::LoadVal(1, 0xAA), // $1 = 0x55
            Instruction::Xor(1, 2),        // $1 = $1 ^ $2 (0xFF)
            Instruction::Xor(1, 1),        // $1 = $1 ^ $1 (0x00)

            Instruction::LoadVal(1, 0x55), // $1 = 0x55
            Instruction::ShiftLeft(1),     // $1 <<= 1 (0xAA)
            Instruction::ShiftLeft(1),     // $1 <<= 1 (0x54 + overflow)

            Instruction::LoadVal(1, 0x55), // $1 = 0x55
            Instruction::ShiftRight(1),    // $1 <<= 1 (0x2A)
            Instruction::ShiftRight(1),    // $1 <<= 1 (0x15 + overflow)
        ]);
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x206);
        assert_eq!(emu.regs[1], 0xFF);

        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x20A);
        assert_eq!(emu.regs[1], 0x00);

        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x20E);
        assert_eq!(emu.regs[1], 0xFF);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x210);
        assert_eq!(emu.regs[1], 0x00);

        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x214);
        assert_eq!(emu.regs[1], 0xAA);
        assert_eq!(emu.regs[0xF], 0);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x216);
        assert_eq!(emu.regs[1], 0x54);
        assert_eq!(emu.regs[0xF], 1);

        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x21A);
        assert_eq!(emu.regs[1], 0x2A);
        assert_eq!(emu.regs[0xF], 1);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x21C);
        assert_eq!(emu.regs[1], 0x15);
        assert_eq!(emu.regs[0xF], 0);
    }

    #[test]
    fn test_013_skip() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_instr(vec![
            Instruction::LoadVal(1, 0xAA),
            Instruction::SkipValEq(1, 0xAB),
            Instruction::SkipValEq(1, 0xAA),
            Instruction::Invalid,

            Instruction::SkipValNotEq(1, 0xAA),
            Instruction::SkipValNotEq(1, 0xAB),
            Instruction::Invalid,

            Instruction::LoadVal(2, 0xAB), // $1 = 0x55
            Instruction::SkipEq(1, 2),
            Instruction::LoadVal(3, 0xAA), // $1 = 0x55
            Instruction::SkipEq(1, 3),
            Instruction::Invalid,
        ]);

        assert_eq!(emu.pc_reg, 0x200);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x202);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x204);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x208);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x20A);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x20E);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x210);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x212);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x214);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x218);
    }

    #[test]
    fn test_014_jump_call_ret() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_instr(vec![
            Instruction::Jump(0x204),
            Instruction::Invalid,
            Instruction::Call(0x208),
            Instruction::Invalid,
            Instruction::Ret,
        ]);
        assert_eq!(emu.pc_reg, 0x200);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x204);
        assert_eq!(emu.sp_reg, 0);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x208);
        assert_eq!(emu.sp_reg, 1);
        assert_eq!(emu.stack[0], 0x206);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x206);
        assert_eq!(emu.sp_reg, 0);
    }

    #[test]
    fn test_015_sprite_draw_cls() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_instr(vec![
            Instruction::LoadVal(1, 0x00),
            Instruction::LoadVal(2, 0x00),
            Instruction::LoadVal(3, 0x0F),

            Instruction::LoadSprite(3),
            Instruction::Draw(1, 2, 5),
             // check overflow
            Instruction::Draw(1, 2, 5),

            // check wrap around
            Instruction::LoadVal(1, 63),
            Instruction::LoadVal(2, 31),
            Instruction::Draw(1, 2, 5),

            // check clear
            Instruction::Cls,

        ]);
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x206);

        emu.cpu_one_cycle();
        assert_eq!(emu.i_reg, 0xF * 5);
        assert_eq!(emu.pc_reg, 0x208);

        // draw letter "F"
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x20A);
        assert_eq!(emu.regs[0xF], 0);
        assert_eq!(emu.screen[0][0], true);
        assert_eq!(emu.screen[1][0], true);
        assert_eq!(emu.screen[2][0], true);
        assert_eq!(emu.screen[3][0], true);
        assert_eq!(emu.screen[4][0], false);
        assert_eq!(emu.screen[0][1], true);
        assert_eq!(emu.screen[1][1], false);
        assert_eq!(emu.screen[0][2], true);
        assert_eq!(emu.screen[1][2], true);
        assert_eq!(emu.screen[2][2], true);
        assert_eq!(emu.screen[3][2], true);
        assert_eq!(emu.screen[4][2], false);
        assert_eq!(emu.screen[0][3], true);
        assert_eq!(emu.screen[1][3], false);
        assert_eq!(emu.screen[0][4], true);
        assert_eq!(emu.screen[1][4], false);

        // clear letter "F"
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x20C);
        assert_eq!(emu.regs[0xF], 1);
        assert_eq!(emu.screen[0][0], false);
        assert_eq!(emu.screen[1][0], false);
        assert_eq!(emu.screen[2][0], false);
        assert_eq!(emu.screen[3][0], false);
        assert_eq!(emu.screen[4][0], false);
        assert_eq!(emu.screen[0][1], false);
        assert_eq!(emu.screen[1][1], false);
        assert_eq!(emu.screen[0][2], false);
        assert_eq!(emu.screen[1][2], false);
        assert_eq!(emu.screen[2][2], false);
        assert_eq!(emu.screen[0][3], false);
        assert_eq!(emu.screen[1][3], false);
        assert_eq!(emu.screen[0][4], false);
        assert_eq!(emu.screen[1][4], false);

        // draw letter F on border
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x212);
        assert_eq!(emu.regs[0xF], 0);
        assert_eq!(emu.screen[63][31], true);
        assert_eq!(emu.screen[0][31], true);
        assert_eq!(emu.screen[1][31], true);
        assert_eq!(emu.screen[2][31], true);
        assert_eq!(emu.screen[3][31], false);
        assert_eq!(emu.screen[63][0], true);
        assert_eq!(emu.screen[0][0], false);
        assert_eq!(emu.screen[63][1], true);
        assert_eq!(emu.screen[0][1], true);
        assert_eq!(emu.screen[1][1], true);
        assert_eq!(emu.screen[2][1], true);
        assert_eq!(emu.screen[3][1], false);
        assert_eq!(emu.screen[63][2], true);
        assert_eq!(emu.screen[0][2], false);
        assert_eq!(emu.screen[63][3], true);
        assert_eq!(emu.screen[0][3], false);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x214);
        assert_eq!(emu.regs[0xF], 0);
        assert_eq!(emu.screen[63][31], false);
        assert_eq!(emu.screen[0][31], false);
        assert_eq!(emu.screen[1][31], false);
        assert_eq!(emu.screen[2][31], false);
        assert_eq!(emu.screen[3][31], false);
        assert_eq!(emu.screen[63][0], false);
        assert_eq!(emu.screen[0][0], false);
        assert_eq!(emu.screen[63][1], false);
        assert_eq!(emu.screen[0][1], false);
        assert_eq!(emu.screen[1][1], false);
        assert_eq!(emu.screen[2][1], false);
        assert_eq!(emu.screen[3][1], false);
        assert_eq!(emu.screen[63][2], false);
        assert_eq!(emu.screen[0][2], false);
        assert_eq!(emu.screen[63][3], false);
        assert_eq!(emu.screen[0][3], false);
    }

    #[test]
    fn test_016_loadaddr_addi_regs_store_load() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_instr(vec![
            Instruction::LoadAddr(0x600),
            Instruction::LoadVal(0, 0xDE),
            Instruction::LoadVal(1, 0xAD),
            Instruction::LoadVal(2, 0xBE),
            Instruction::LoadVal(3, 0xEF),
            Instruction::StoreRegs(2),

            Instruction::LoadVal(0, 0x00),
            Instruction::LoadVal(1, 0x00),
            Instruction::LoadVal(2, 0x00),
            Instruction::LoadRegs(2),
            Instruction::AddI(1),
        ]);
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x202);
        assert_eq!(emu.i_reg, 0x600);

        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x20A);

        emu.cpu_one_cycle();
        assert_eq!(emu.memory[0x600], 0xDE);
        assert_eq!(emu.memory[0x601], 0xAD);
        assert_eq!(emu.memory[0x602], 0xBE);
        assert_eq!(emu.memory[0x603], 0x00);

        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x212);

        emu.cpu_one_cycle();
        assert_eq!(emu.regs[0], 0xDE);
        assert_eq!(emu.regs[1], 0xAD);
        assert_eq!(emu.regs[2], 0xBE);
        assert_eq!(emu.regs[3], 0xEF);
        assert_eq!(emu.pc_reg, 0x214);

        emu.cpu_one_cycle();
        assert_eq!(emu.i_reg, 0x6AD);
    }

    #[test]
    fn test_017_bcd() {
        let mut emu = Emulator::new(DisplaySize::Basic64x32);
        emu.mem_load_instr(vec![
            Instruction::LoadVal(1, 234),
            Instruction::LoadAddr(0x600),
            Instruction::Bcd(1),
        ]);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x202);
        assert_eq!(emu.regs[1], 234);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x204);
        assert_eq!(emu.i_reg, 0x600);

        emu.cpu_one_cycle();
        assert_eq!(emu.pc_reg, 0x206);
        assert_eq!(emu.memory[0x600], 2);
        assert_eq!(emu.memory[0x601], 3);
        assert_eq!(emu.memory[0x602], 4);
    }
}
