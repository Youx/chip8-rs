mod ui_pixels;

use emulator::Emulator;
use emulator::DisplaySize;
use emulator::ui::Screen;
use ui_pixels::UIPixels;

use std::env;
use std::path::Path;
use std::{io::Read, fs::File};

fn main() {
    let mut emu : Emulator = Emulator::new(DisplaySize::Basic64x32);
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => panic!("pass an argument"),
        2 => {
            let mut rom = [0; 4096 - 0x200];
            let path = Path::new(&args[1]);
            let mut file = match File::open(&path) {
                Ok(file) => file,
                Err(reason) => panic!("failed to open file: {}", reason),
            };
            match file.read(&mut rom) {
                Ok(_) => {
                    emu.mem_load_bin(rom.to_vec());
                },
                Err(reason) => panic!("failed to read file: {}", reason)
            }
        },
        _ => panic!("too many arguments"),
    }
    let ui = UIPixels::new(emu);
    ui.run();
}
