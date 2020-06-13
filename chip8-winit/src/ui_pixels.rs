use pixels::{wgpu::Surface, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use emulator::ui::Screen;
use emulator::Emulator;
use std::time::{Duration, SystemTime};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 64;

pub struct UIPixels {
    emu: Emulator,
}

impl Screen for UIPixels {
    fn new(emu: Emulator) -> Self {
        UIPixels {
            emu
        }
    }
    fn run(mut self) {
        let keys = vec![
            VirtualKeyCode::Key0,
            VirtualKeyCode::Key1,
            VirtualKeyCode::Key2,
            VirtualKeyCode::Key3,
            VirtualKeyCode::Key4,
            VirtualKeyCode::Key5,
            VirtualKeyCode::Key6,
            VirtualKeyCode::Key7,
            VirtualKeyCode::Key8,
            VirtualKeyCode::Key9,
            VirtualKeyCode::A,
            VirtualKeyCode::B,
            VirtualKeyCode::C,
            VirtualKeyCode::D,
            VirtualKeyCode::E,
            VirtualKeyCode::F,
        ];
        let event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();
        let window = {
            let size = LogicalSize::new((WIDTH * 10) as f64, (HEIGHT * 10) as f64);
            WindowBuilder::new()
                .with_title("Hello Pixels")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let mut pixels = {
            let surface = Surface::create(&window);
            let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, surface);
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        let mut now = SystemTime::now();
        let mut instruction_count = 0;
        let mut draw_count = 0;
        event_loop.run(move |event, _, control_flow| {
            if let Ok(val) = now.elapsed() {
                if val > Duration::from_secs(1) {
                    println!("{} ips, {} dps", instruction_count, draw_count);
                    now = SystemTime::now();
                    instruction_count = 0;
                    draw_count = 0;
                }
            }
            self.emu.cpu_one_cycle();
            instruction_count += 1;
            //if self.emu.redraw {
            //    window.request_redraw();
            //    self.emu.redraw = false;
            //}
            if instruction_count % 30 == 0 {
                window.request_redraw();
            }

            if let Event::RedrawRequested(_) = event {
                let frame = pixels.get_frame();

                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let x = (i % WIDTH as usize) as usize;
                    let y = (i / WIDTH as usize) as usize;

                    if x < self.emu.resolution.0 && y < self.emu.resolution.1 {
                        if self.emu.screen[x][y] {
                            pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0xFF]);
                        } else {
                            pixel.copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
                        }
                    }
                }
                draw_count += 1;
                if pixels
                .render()
                .map_err(|e| panic!("pixels.render() failed: {}", e))
                .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            if input.update(event) {
                if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                for (idx, k) in keys.iter().enumerate() {
                    if self.emu.keys[idx] && input.key_released(*k) {
                        //println!("key released: {:?}", *k);
                        self.emu.keys[idx] = false;
                    } else if !self.emu.keys[idx] && input.key_pressed(*k) {
                        //println!("key pressed: {:?}", *k);
                        self.emu.keys[idx] = true;
                    }
                }
            }
        });
    }
}
