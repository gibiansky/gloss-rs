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
    let mut window = gloss_core::GlossWindow::new(400, 400, "Gloss Demo", Color::Black);
    let mut picture = Pictures(vec![
                               Colored(Color::RGB(0.0, 0.6, 0.8), box Circle(200.0)),
                               Colored(Color::RGB(0.0, 0.7, 0.9), box Circle(100.0)),
                               Colored(Color::White, box Line(vec![point(-100.0, -100.0), point(100.0, 100.0)])),
                               ]);

    while !window.done() {
        window.draw(&picture);
        window.update(|event| update_picture(&mut picture, event));
    }
}

fn update_picture(picture: &mut Picture, event: event::Event) {
}
