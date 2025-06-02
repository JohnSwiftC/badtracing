use minifb::{Window, WindowOptions};

const WINDOW_W: usize = 1000;
const WINDOW_H: usize = 1000;

fn main() {
    let mut window = Window::new(
        "badtracing",
        WINDOW_W,
        WINDOW_H,
        WindowOptions::default()
    ).expect("Window failed to open.");

    let red = from_u8_rgb(255, 0, 0);
    let buffer: Vec<u32> = vec![red; WINDOW_H * WINDOW_W];

    // Keep window open
    loop {
        window.update_with_buffer(&buffer, WINDOW_W, WINDOW_H).unwrap();
    }
}

// Ripped straight from the docs lol
fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}