use minifb::{Window, WindowOptions};

const WINDOW_W: usize = 1000;
const WINDOW_H: usize = 1000;
const FPS: usize = 30;
const FOCAL_DISTANCE: u32 = 1;
const VIEWPORT_SIZE: u32 = 1; // Width of the viewport used for calculations
const BACKGROUND_COLOR: u32 = 0;
const RAY_FINENESS: f32 = 2.0; // How much the dx and dy are divided by for each step in the raycast. Higher values lead to more accurate casts but slower performance

struct Player {
    position: Position,
    view_angle: i32, // Principal axis is facing down, deviation is in radians.
}

impl Player {
    fn new() -> Self {
        Self {
            position: Position {
                x: 0,
                y: 0,
            },
            view_angle: 0,
        }
    }

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

/// Buffer with an x, y coordinate system that allows for easy, specific updates.
/// Includes a method to convert to normal screen buffer
struct Buffer2D(Vec<Vec<u32>>);

impl Buffer2D {
    fn new(width: usize, height: usize) -> Self {
        Self(vec![vec![0; height]; width])
    }

    /// Does this in-place to an existing screen buffer
    /// Also hoping that the buffer is the same size as the Buffer2D
    fn to_screen(self, buffer: &mut Vec<u32>) {
        let mut x = 0;
        for i in 0..self.0.len() {
            for k in 0..self.0[0].len() { // super readable
                buffer[x] = self.0[i][k];
                x += 1;
            }
        }
    }
}

fn main() {

    let mut window = Window::new(
        "badtracing",
        WINDOW_W,
        WINDOW_H,
        WindowOptions::default()
    ).expect("Window failed to open.");
    window.set_target_fps(FPS);

    let mut buffer = vec![BACKGROUND_COLOR; WINDOW_H * WINDOW_W];

    let map = [
        [0, 0, 0, 0, 1, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0, 0, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        [0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let mut player = Player::new();

    // Main loop
    let column_width: f32 = (VIEWPORT_SIZE / WINDOW_W as u32) as f32;
    loop {
        // Loop for each column in the screen, cast ray for each
        for c in 0..WINDOW_W {
            if c < WINDOW_W / 2 {
                let dx = ((VIEWPORT_SIZE / 2) as f32 - column_width * c as f32) / RAY_FINENESS;
                let dy = (FOCAL_DISTANCE as f32) / RAY_FINENESS;

                let mut ray_x = player.position.x as f32;
                let mut ray_y = player.position.y as f32;

                while ray_x <= map.len() as f32 - 1.0 || ray_x > 0.0 || ray_y <= map[0].len() as f32 - 1.0 || ray_y > 0.0 { // Map bounds checks
                    if map[ray_x.floor() as usize][ray_y.floor() as usize] == 1 {
                        let dist = (ray_x * ray_x + ray_y * ray_y).sqrt();
                    }
                }
            }
        }
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

fn flush_buffer(buffer: &mut Vec<u32>) {
    buffer.iter_mut().for_each(|i| *i = BACKGROUND_COLOR);
}

fn draw_line(buffer: &mut Buffer2D, h: u32, c: usize, color: u32) {
    let offset = (WINDOW_H - h as usize) / 2;
    for i in offset..offset + h as usize {
        buffer.0[c][i] = color;
    }
}