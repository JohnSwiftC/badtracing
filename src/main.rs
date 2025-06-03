use minifb::{Window, WindowOptions};

const WINDOW_W: usize = 1000;
const WINDOW_H: usize = 1000;
const FPS: usize = 30;

fn main() {
    let mut window = Window::new(
        "badtracing",
        WINDOW_W,
        WINDOW_H,
        WindowOptions::default()
    ).expect("Window failed to open.");

    //window.set_target_fps(FPS);

    let mut red = from_u8_rgb(255, 50, 120);
    let blue = from_u8_rgb(0, 0, 255);
    let mut red_buffer: Vec<u32> = vec![red; WINDOW_H * WINDOW_W];

    window.set_target_fps(30);

    // Keep window open
    loop {
        window.update_with_buffer(&red_buffer, WINDOW_W, WINDOW_H).expect("Failed to update window buffer");
        red = decrease_brightness(red, 1);
        red_buffer = vec![red; WINDOW_H * WINDOW_W];
    }
}

// Ripped straight from the docs lol
fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn decrease_brightness(color: u32, amount: u32) -> u32 {
    let mut r = color >> 16;
    let mut g = (color - (r << 16)) >> 8;
    let mut b = color - (r << 16) - (g << 8);

    if amount <= r { r -= amount; }
    if amount <= g { g -= amount; }
    if amount <= b { b -= amount; }

    (r << 16) | (g << 8) | b
}

fn increase_brightness(color: u32, amount: u32) -> u32 {
    let mut r = color >> 16;
    let mut g = (color - (r << 16)) >> 8;
    let mut b = color - (r << 16) - (g << 8);

    if r + amount <= 255 { r += amount; }
    if g + amount <= 255 { g += amount; }
    if b + amount <= 255 { b += amount; }

    (r << 16) | (g << 8) | b
}