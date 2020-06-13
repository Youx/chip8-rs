use emulator::ui::Screen;
use emulator::Emulator;

pub struct Terminal {
    emu: Emulator,
}
impl Screen for Terminal {
    fn new(emu: Emulator) -> Self {
        Terminal { emu }
    }
    fn run(mut self) {
        loop {
            self.emu.cpu_one_cycle();
            if self.emu.redraw {
                print!("\x1B[{};{}H", 1, 1);
                print!("\u{250C}");
                for _ in 0..self.emu.resolution.0 {
                    print!("\u{2500}");
                }
                println!("\u{2510}");


                for y in 0..self.emu.resolution.1 {
                    print!("\u{2502}");
                    for x in 0..self.emu.resolution.0 {
                        print!("{}", if self.emu.screen[x][y] { "*" } else { " " });
                    }
                    println!("\u{2502}");
                }

                print!("\u{2514}");
                for _ in 0..self.emu.resolution.0 {
                    print!("\u{2500}");
                }
                println!("\u{2518}");
            }
        }
    }
}
