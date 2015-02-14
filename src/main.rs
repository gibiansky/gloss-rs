#![feature(box_syntax)]
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
    let mut window = gloss_core::GlossWindow::new(400, 400, "Gloss Demo", Color::Black);
    let pts = vec![point(-150.0, -150.0), point(0.0, 150.0), point(150.0, -150.0)];
    let mut picture = Colored(Color::Red, box Circle(200.0));
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
