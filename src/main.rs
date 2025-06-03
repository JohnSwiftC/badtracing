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

    let red = from_u8_rgb(255, 0, 0);
    let blue = from_u8_rgb(0, 0, 255);
    let red_buffer: Vec<u32> = vec![red; WINDOW_H * WINDOW_W];
    let blue_buffer: Vec<u32> = vec!(blue; WINDOW_H * WINDOW_W);

    // Keep window open
    let mut is_red = false;
    loop {
        if is_red {
            window.update_with_buffer(&blue_buffer, WINDOW_W, WINDOW_H).unwrap();
            is_red = false;
        } else {
            window.update_with_buffer(&red_buffer, WINDOW_W, WINDOW_H).unwrap();
            is_red = true;
        }
    }
}

// Ripped straight from the docs lol
fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}