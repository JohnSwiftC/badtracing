use minifb::{Window, WindowOptions};

const WINDOW_W: usize = 1000;
const WINDOW_H: usize = 1000;
const FPS: usize = 30;
const FOCAL_DISTANCE: u32 = 1;

struct Player {
    position: Position,
    view_angle: i32, // Principal axis is facing down, deviation is in radians.
}

impl Player {
    /// Updates absolute position
    fn set_position(&mut self, x: i32, y: i32) {
        self.position.x = x;
        self.position.y = y;
    }

    /// Updates relative position
    fn update_position(&mut self, x: i32, y: i32) {
        self.position.x += x;
        self.position.y += y;
    }

    /// Updates absolute angle
    fn set_angle(&mut self, theta: i32) {
        self.view_angle = theta;
    }

    /// Updates relative angle
    fn update_angle(&mut self, theta: i32) {
        self.view_angle += theta;
    }
}

struct Position {
    x: i32,
    y: i32, // this is a 2d x,y coordinate plane
}

fn main() {
    let mut window = Window::new(
        "badtracing",
        WINDOW_W,
        WINDOW_H,
        WindowOptions::default()
    ).expect("Window failed to open.");

    let mut red = from_u8_rgb(255, 50, 120);
    let blue = from_u8_rgb(0, 0, 255);
    let mut red_buffer: Vec<u32> = vec![red; WINDOW_H * WINDOW_W];

    window.set_target_fps(FPS);

    let map = [
        [0, 0, 0, 0, 1, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0, 0, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        [0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    // Keep window open
    loop {
        window.update_with_buffer(&red_buffer, WINDOW_W, WINDOW_H).expect("Failed to update window buffer");
        red = decrease_brightness(red, 5);
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
    let mut g = (color >> 8) & 255;
    let mut b = color & 255;

    if amount <= r { r -= amount; } else { r = 0; }
    if amount <= g { g -= amount; } else { g = 0; }
    if amount <= b { b -= amount; } else { b = 0; }

    (r << 16) | (g << 8) | b
}

fn increase_brightness(color: u32, amount: u32) -> u32 {
    let mut r = color >> 16;
    let mut g = (color >> 8) & 255;
    let mut b = color & 255;

    if r + amount <= 255 { r += amount; } else { r = 255; }
    if g + amount <= 255 { g += amount; } else { g = 255; }
    if b + amount <= 255 { b += amount; } else { b = 255; }

    (r << 16) | (g << 8) | b
}