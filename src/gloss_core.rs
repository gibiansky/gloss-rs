extern crate glfw;
extern crate gl;

use gl::types::*;
use glfw::Context;


use picture::*;
use picture::Picture::*;
use event::*;
use event::Event::*;

use std::mem;
use std::ptr;
use std::ffi::CString;
use std::str;
use std::sync::mpsc::Receiver;


// Shader sources
static POLYGON_VERTEX_SHADER: &'static str =
"\
#version 330\n\
layout(location = 0) in vec2 position;\n\
void main() {\n\
    gl_Position = vec4(position, 0.0, 1.0);\n\
}\
";

static COLOR_FRAGMENT_SHADER: &'static str =
"\
#version 330\n\
uniform vec3 color;\n\
out vec4 out_color;\n\
void main() {\n\
    out_color = vec4(0.5, 0.5, 1.0, 1.0);\n\
}\
";

pub struct GlossWindow {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    vao: GLuint,
    vbo: GLuint,
}

impl GlossWindow {
    pub fn new(width: u32, height: u32, title: &str, background: Color) -> GlossWindow {
        unsafe {
            // Initialize GLFW.
            let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

            glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
            glfw.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
            glfw.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));

            // Create the window, dying if it fails.
            let (mut window, events) = glfw.create_window(width, height, title, glfw::WindowMode::Windowed)
                .expect("Failed to create GLFW window.");

            // Start using the window, responding to any events.
            window.set_key_polling(true);
            window.make_current();

            gl::load_with(|s| window.get_proc_address(s));

            let mut win = GlossWindow {
                glfw: glfw,
                window: window,
                events: events,
                vao: 0,
                vbo: 0,
            };

            // Create Vertex Array Object, and set it as the current one
            gl::GenVertexArrays(1, &mut win.vao);
            gl::BindVertexArray(win.vao);

            // Create GLSL shaders
            let vs = compile_shader(POLYGON_VERTEX_SHADER, gl::VERTEX_SHADER);
            let fs = compile_shader(COLOR_FRAGMENT_SHADER, gl::FRAGMENT_SHADER);
            let program = link_program(vs, fs);

            // Create a Vertex Buffer Object.
            gl::GenBuffers(1, &mut win.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, win.vbo);

            // Link attribute 0 (position).
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, win.vbo);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

            // Use our shader program
            gl::UseProgram(program);

            win
        }
    }

    pub fn draw(&mut self, picture: &Picture) {
         // Clear the screen to black
        unsafe {
            gl::ClearColor(0.3, 0.0, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            // Draw a triangle from the 3 vertices
            self.draw_picture(picture);
        }

        self.window.swap_buffers();
    }

    pub fn done(&self) -> bool {
        return self.window.should_close();
    }

    fn fill_vertex_buffer(&self, pts: &Points) {
        unsafe {
            // Write data into our buffer.
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (pts.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                           mem::transmute(pts.as_ptr()),
                           gl::STATIC_DRAW);
        }
    }

    fn draw_picture(&self, picture: &Picture) {
        match *picture {
            Blank => (),
            Polygon(ref pts) => {
                unsafe {
                    self.fill_vertex_buffer(&pts);
                    gl::DrawArrays(gl::TRIANGLE_FAN, 0, pts.len() as i32);
                }
            },
            _ => println!("Not implemented!"),
        }
    }

    pub fn close(&mut self) {
        self.window.set_should_close(true);
    }

    pub fn update<F: FnMut(Event) -> ()>(&mut self, mut handler: F) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            let gloss_event = glfw_event_to_gloss(event);
            handler(gloss_event);
        }
    }
}

fn glfw_event_to_gloss(event: glfw::WindowEvent) -> Event {
    match event {
        glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => KeyPress,
        _ => MousePress
    }
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        // Make a new shader of a given type.
        shader = gl::CreateShader(ty);

        // Attempt to compile the shader.
        let c_str = CString::from_slice(src.as_bytes());
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status.
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buf: Vec<u8> = Vec::with_capacity((len as usize) - 1);
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("Failed shader compilation. {}",
                   str::from_utf8(buf.as_slice()).ok().expect("ShaderInfoLog not valid utf8"));
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

            let mut buf: Vec<u8> = Vec::with_capacity((len as usize) - 1);
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("Failed program linking. {}",
                   str::from_utf8(buf.as_slice()).ok().expect("ProgramInfoLog not valid utf8"));
        }

        program
    }
}
