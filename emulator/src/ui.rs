use crate::Emulator;

pub trait Screen {
    fn new(emu: Emulator) -> Self;
    fn run(self);
}
