use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    WebGlShader,
    WebGlProgram,
};
use web_sys::WebGlRenderingContext;
use js_sys::{
    Uint8Array
};

use emulator::Emulator;
use emulator::DisplaySize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
mod shaders;

#[wasm_bindgen]
pub struct UICanvas {
    emu: Emulator,
    context: WebGlRenderingContext,
}

fn perf_to_system(amt: f64) -> SystemTime {
    let secs = (amt as u64) / 1_000;
    let nanos = ((amt as u32) % 1_000) * 1_000_000;
    UNIX_EPOCH + Duration::new(secs, nanos)
}

#[wasm_bindgen]
pub fn start(data: &JsValue) -> UICanvas {
    console_error_panic_hook::set_once();

    let data = Uint8Array::new(&data);
    let mut rom_bin: Vec<u8> = vec![0; data.length() as usize];
    data.copy_to(&mut rom_bin);

    let window = web_sys::window().expect("global window does not exists");

    let performance = window
        .performance()
        .expect("performance should be available");
    let mut emu = Emulator::new_with_time(DisplaySize::Basic64x32,
        perf_to_system(performance.now()));
    emu.mem_load_bin(rom_bin);
    let renderer = UICanvas::new(emu);

    renderer
}

fn compile_shader(gl: &WebGlRenderingContext, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Error creating shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
            Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader)
              .unwrap_or_else(||
                String::from("Unable to get shader info log"))
        )
    }           
}

fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
    .create_program()
    .ok_or_else(|| String::from("Unable to create webgl program"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

#[wasm_bindgen]
impl UICanvas {
    pub fn set_key_pressed(&mut self, val: usize) {
        self.emu.keys[val] = true;
    }
    pub fn set_key_released(&mut self, val: usize) {
        self.emu.keys[val] = false;
    }
    fn new(emu: Emulator) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").expect("canvas not found");
        let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

        canvas.set_width(640);
        canvas.set_height(640);

        let context = canvas
            .get_context("webgl").expect("Failed to get webgl context")
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()
            .unwrap();

        context.enable(WebGlRenderingContext::BLEND);
        context.blend_func(WebGlRenderingContext::SRC_ALPHA, WebGlRenderingContext::ONE_MINUS_SRC_ALPHA);
        context.clear_color(0.0, 0.0, 0.0, 1.0); // black
        context.clear_depth(1.0);
        let vertex_shader = compile_shader(
            &context,
            WebGlRenderingContext::VERTEX_SHADER,
            shaders::vertex::SHADER
        ).unwrap();
        let frag_shader = compile_shader(
            &context,
            WebGlRenderingContext::FRAGMENT_SHADER,
            shaders::fragment::SHADER
        ).unwrap();
        let program = link_program(&context, &vertex_shader, &frag_shader).unwrap();
        context.use_program(Some(&program));

        UICanvas {
            emu,
            context,
        }
    }
    pub fn reload(&mut self, data: &JsValue) {
        let data = Uint8Array::new(&data);
        let mut rom_bin: Vec<u8> = vec![0; data.length() as usize];
        data.copy_to(&mut rom_bin);
        self.emu.reset();
        self.emu.mem_load_bin(rom_bin);
    }
    pub fn run(&mut self) {
        let window = web_sys::window().expect("global window does not exists");

        let performance = window
            .performance()
            .expect("performance should be available");

        for _ in 0..10 {
            self.emu.cpu_one_cycle_with_time(perf_to_system(performance.now()));
        }
        if self.emu.redraw {
            self.emu.redraw = false;
            let buffer = self.context.create_buffer().ok_or("failed to create buffer").unwrap();
            self.context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

            let mut vertices: Vec<f32> = Vec::new();
            for x in 0..self.emu.resolution.0 {
                let x_f = x as f32 / 64.0 * 2.0 - 1.0;
                let xnext_f = (x as f32 + 1.0) / 64.0 * 2.0 - 1.0;

                for y in 0..self.emu.resolution.1 {
                    if self.emu.screen[x][y] {
                        let y_f = (64.0 - y as f32) / 64.0 * 2.0 - 1.0;
                        let ynext_f = (64.0 - (y as f32 + 1.0)) / 64.0 *2.0 - 1.0;

                        let mut rect = vec![
                            x_f, y_f, 0.0,
                            xnext_f, y_f, 0.0,
                            xnext_f, ynext_f, 0.0,

                            xnext_f, ynext_f, 0.0,
                            x_f, ynext_f, 0.0,
                            x_f, y_f, 0.0,
                        ];

                        vertices.append(&mut rect);
                    }
                }
            }

            unsafe {
                let vert_array = js_sys::Float32Array::view(&vertices);
        
                self.context.buffer_data_with_array_buffer_view(
                    WebGlRenderingContext::ARRAY_BUFFER,
                    &vert_array,
                    WebGlRenderingContext::STATIC_DRAW,
                );
            }

            self.context.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
            self.context.enable_vertex_attrib_array(0);

            self.context.clear_color(0.0, 0.0, 0.0, 1.0);
            self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

            self.context.draw_arrays(
                WebGlRenderingContext::TRIANGLES,
                0,
                (vertices.len() / 3) as i32,
            );
        }
    }
}
