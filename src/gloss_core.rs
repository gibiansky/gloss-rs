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
use std::num::Float;
use std::f32::consts::PI_2;


// Vertex shader source.
// This vertex shader takes two floats and treats them as the x and y coordinates of the resulting
// point. It scales them by 1/width or 1/height, respectively, which it obtains from the "size"
// uniform two-float vector.
static POLYGON_VERTEX_SHADER: &'static str =
"\
#version 330\n\
layout(location = 0) in vec2 position;\n\
uniform vec2 size;\n\
void main() {\n\
    gl_Position = vec4(position.x / size.x, position.y / size.y, 0.0, 1.0);\n\
}\
";

// Fragment shader source.
// This fragment shader simply sets the color to whatever it is given via the "color" uniform.
static COLOR_FRAGMENT_SHADER: &'static str =
"\
#version 330\n\
out vec4 out_color;\n\
uniform vec4 color;\n\
void main() {\n\
    out_color = color;
}\
";

static mut initialized: bool = false;

pub struct GlossWindow {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    vao: GLuint,
    vbo: GLuint,
    background: Color,
    color_uniform: GLint,
    program: GLuint,
}

impl GlossWindow {
    /// Create a new window in which to render.
    /// This also initializes the GLFW and OpenGL backends, so this should be done only once and on
    /// the main thread. Calling this a second time will result in an error.
    pub fn new(width: u32, height: u32, title: &str, background: Color) -> GlossWindow {
        unsafe {
            // Only one Gloss window permitted.
            if initialized {
                panic!("GlossWindow::new invoked twice – only once allowed.");
            }
            initialized = true;

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

            // Load OpenGL function pointers.
            gl::load_with(|s| window.get_proc_address(s));

            // Create our window. This stores anything that Gloss needs for later rendering.
            let mut win = GlossWindow {
                glfw: glfw,
                window: window,
                events: events,
                vao: 0,
                vbo: 0,
                background: background,
                color_uniform: 0,
                program: 0,
            };

            // Create Vertex Array Object, and set it as the current one.
            gl::GenVertexArrays(1, &mut win.vao);
            gl::BindVertexArray(win.vao);

            // Create GLSL shaders.
            let vs = compile_shader(POLYGON_VERTEX_SHADER, gl::VERTEX_SHADER);
            let fs = compile_shader(COLOR_FRAGMENT_SHADER, gl::FRAGMENT_SHADER);
            win.program = link_program(vs, fs);

            // Create a Vertex Buffer Object.
            gl::GenBuffers(1, &mut win.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, win.vbo);

            // Link attribute 0 (position).
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, win.vbo);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());

            // Use our shader program.
            gl::UseProgram(win.program);

            // Set the size uniform for the vertex shader.
            let size_uniform = gl::GetUniformLocation(win.program, CString::from_slice("size".as_bytes()).as_ptr());
            gl::Uniform2f(size_uniform, width as GLfloat, height as GLfloat);

            // Set the default color for drawing shapes to white.
            win.color_uniform = gl::GetUniformLocation(win.program, CString::from_slice("color".as_bytes()).as_ptr());
            gl::Uniform4f(win.color_uniform, 1.0, 1.0, 1.0, 1.0);

            win
        }
    }

    pub fn draw(&mut self, picture: &Picture) {
        unsafe {
            // Clear the screen to whatever background color it has.
            let (r, g, b, a) = color_to_rgba(self.background);
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
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
                           (pts.len() * 2 * mem::size_of::<GLfloat>()) as GLsizeiptr,
                           mem::transmute(pts.as_ptr()),
                           gl::STATIC_DRAW);
        }
    }

    fn draw_picture(&self, picture: &Picture) {
        match *picture {
            // Simplest possible picture: nothing.
            Blank => (),

            // Shapes.
            Polygon(ref pts) => {
                unsafe {
                    self.fill_vertex_buffer(&pts);
                    gl::DrawArrays(gl::TRIANGLE_FAN, 0, pts.len() as i32 * 2);
                }
            },
            Circle(radius) => {
                unsafe {
                    let pts = circle_to_polygon(radius, 50);
                    self.fill_vertex_buffer(&pts);
                    gl::DrawArrays(gl::TRIANGLE_FAN, 0, pts.len() as i32 * 2);
                }
            },

            // Lines.
            Line(ref pts) => {
                unsafe {
                    gl::Enable(gl::LINE_SMOOTH);
                    self.fill_vertex_buffer(&pts);
                    gl::DrawArrays(gl::LINE_STRIP, 0, pts.len() as i32 * 2);
                }
            }

            // Transforms.
            Colored(color, ref pic) => {
                let (r, g, b, a) = color_to_rgba(color);
                unsafe {
                    let mut buf: Vec<GLfloat> = Vec::with_capacity(4);
                    buf.set_len(4);
                    gl::GetUniformfv(self.program, self.color_uniform, buf.as_mut_ptr() as *mut GLfloat);
                    gl::Uniform4f(self.color_uniform, r, g, b, a);
                    self.draw_picture(&pic);
                    gl::Uniform4f(self.color_uniform, buf[0], buf[1], buf[2], buf[3]);
                }
            },

            // Combine many pictures. Draw them in the order they appear.
            Pictures(ref pics) => {
                for pic in pics {
                    self.draw_picture(&pic);
                }
            }

            // FIXME: Remove this once all pieces of Picture are implemented.
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
            buf.set_len(len as usize - 1);
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
            buf.set_len(len as usize - 1);
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("Failed program linking. {}",
                   str::from_utf8(buf.as_slice()).ok().expect("ProgramInfoLog not valid utf8"));
        }

        program
    }
}

fn circle_to_polygon(radius: f32, n_points: usize) -> Points {
    let mut pts = Vec::with_capacity(n_points + 1);

    // The center is just at zero
    pts.push(point(0.0, 0.0));

    for i in range(0, n_points + 1) {
        let f = i as f32 / n_points as f32;
        let x = radius * (f * PI_2).cos();
        let y = radius * (f * PI_2).sin();
        pts.push(point(x, y));
    }

    pts
}
