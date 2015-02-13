extern crate gl;
extern crate glfw;

mod picture;
use picture::*;
use picture::Picture::*;

mod event;
use event::*;

mod gloss_core;

fn main() {
    println!("Trying to open");
    let mut window = gloss_core::GlossWindow::new(300, 300, "Gloss Demo", Color::Black);
    let pts = vec![point(0.5, 0.5), point(1.0, 1.0), point(1.5, 0.0)];
    let mut picture = Polygon(pts);
    println!("Opened");

    while !window.done() {
        println!("In loop");
        window.draw(&picture);
        println!("Drawn");
        window.update(|event| update_picture(&mut picture, event));
        println!("Updated");
    }
}

fn update_picture(picture: &mut Picture, event: event::Event) {
}
