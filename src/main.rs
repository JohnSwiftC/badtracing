use minifb::{Window, WindowOptions, Key};
use image::{GenericImageView, DynamicImage};

const WINDOW_W: usize = 1000;
const WINDOW_H: usize = 700;
const FPS: usize = 60;
const FOCAL_DISTANCE: f32 = WINDOW_H as f32 / WINDOW_W as f32;
const VIEWPORT_SIZE: f32 = 1.0; // Width of the viewport used for calculations
const BACKGROUND_COLOR: u32 = 0;
const RAY_FINENESS: f32 = 100.0; // How much the dx and dy are divided by for each step in the raycast. Higher values lead to more accurate casts but slower performance
const HEIGHT_ADJUSTMENT: f32 = 0.3; // Higher values lead to lower heights.
const SHADOW_ADJUSTMENT: f32 = 5.0; // Scales distance to the amount of brightness removed
const PLAYER_VELOCITY: f32 = 0.04; // Scales the movement amount determined by the sin and cosine
const LOOK_SENSE: f32 = 0.02; // Speed of rotation with arrow keys

struct Player {
    position: Position,
    view_angle: f32, // Principal axis is facing right, deviation is in radians.
}

impl Player {
    fn new() -> Self {
        Self {
            position: Position { x: 0.0, y: 0.0 },
            view_angle: 0.0,
        }
    }

    /// Updates absolute position
    fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    /// Updates relative position
    fn update_position(&mut self, x: f32, y: f32) {
        self.position.x += x;
        self.position.y += y;
    }

    /// Updates relative position with collision detection
    fn update_position_checked(&mut self, x: f32, y: f32, map: &Vec<Vec<u8>>) {
        let new_x = self.position.x + x;
        let new_y = self.position.y + y;

        if map[new_y.floor() as usize][self.position.x.floor() as usize] == 1 {
            self.position.x = new_x;
            return;
        }

        if map[self.position.y.floor() as usize][new_x.floor() as usize] == 1 {
            self.position.y = new_y;
            return;
        }

        if map[new_y.floor() as usize][new_x.floor() as usize] == 1 {
            return;
        }

        self.position.x = new_x;
        self.position.y = new_y;
    }

    /// Updates absolute angle
    fn set_angle(&mut self, theta: f32) {
        self.view_angle = theta;
    }

    /// Updates relative angle
    fn update_angle(&mut self, theta: f32) {
        self.view_angle += theta;
    }
}

struct Position {
    x: f32,
    y: f32, // this is a 2d x,y coordinate plane
}

/// Buffer with an x, y coordinate system that allows for easy, specific updates.
/// Includes a method to convert to normal screen buffer
struct Buffer2D(Vec<Vec<u32>>);

impl Buffer2D {
    fn new(height: usize, width: usize) -> Self {
        Self(vec![vec![BACKGROUND_COLOR; height]; width])
    }

    /// Does this in-place to an existing screen buffer
    /// Also hoping that the buffer is the same size as the Buffer2D
    fn to_screen(&self, buffer: &mut [u32]) {
        let mut idx = 0;
        for y in 0..self.0[0].len() {
            for x in 0..self.0.len() {
                buffer[idx] = self.0[x][y];
                idx += 1;
            }
        }
    }

    fn flush(&mut self) {
        for i in 0..self.0.len() {
            for k in 0..self.0[0].len() {
                self.0[i][k] = BACKGROUND_COLOR;
            }
        }
    }
}

use std::path::Path;

struct Skybox {
    image: DynamicImage,
    width: u32,
    height: u32,
}

impl Skybox {
    
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        
        Ok(Skybox {
            image,
            width,
            height,
        })
    }
    
    // Get pixel color from skybox based on viewing angle and vertical position
    fn get_pixel(&self, angle: f32, vertical_ratio: f32) -> u32 {
    
        let two_pi = 2.0 * std::f32::consts::PI;
        let normalized_angle = (angle % (two_pi) + two_pi) / two_pi;
        let u = (normalized_angle * self.width as f32) as u32 % self.width;
        
        let mut v = ((1.0 - vertical_ratio.clamp(0.0, 1.0)) * self.height as f32) as u32;
        v = v.min(self.height - 1);
        
        let pixel = self.image.get_pixel(u, v);
        
        from_u8_rgb(pixel[0], pixel[1], pixel[2])
    }
}

struct Texture {
    image: DynamicImage,
    width: u32,
    height: u32,
}

impl Texture {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        
        Ok(Texture {
            image,
            width,
            height,
        })
    }

    fn get_pixel_uv(&self, u: f32, v: f32) -> u32 {
        // U is relative to x, v is relative to y here
        // I'm using uv because a size of a wall is 1, so we can easily calculate uv with a ray position and wall corner position

        let x = (u * self.width as f32).floor() as u32;
        let y = (v * self.height as f32).floor() as u32;

        // Also, I know that I only need to draw columns so this can be heavily optimized but just poc for now

        let pixel = self.image.get_pixel(x, y);

        from_u8_rgb(pixel[0], pixel[1], pixel[2])

    }
}

enum WorldObject {
    BrickWall,
    RedWall,
}

fn main() {
    let red = from_u8_rgb(255, 0, 0);
    let blue = from_u8_rgb(0, 0, 255);

    let mut window = Window::new("badtracing", WINDOW_W, WINDOW_H, WindowOptions::default())
        .expect("Window failed to open.");
    window.set_target_fps(FPS);

    let mut buffer = Buffer2D::new(WINDOW_H, WINDOW_W);
    let mut screen_buffer = vec![BACKGROUND_COLOR; WINDOW_H * WINDOW_W];

    let map: Vec<Vec<u8>> = vec![
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1],
        vec![1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1],
        vec![1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
        vec![1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        
    ];
    
    let skybox = Skybox::load_from_file("skybox.jpg").expect("skybox failed to load");
    let wall_texture = Texture::load_from_file("wall.jpg").expect("wall texture failed to load");
    
    // Floor is a static gradient, calculating it only once adds a little performance
    let mut floor = Buffer2D::new(WINDOW_H, WINDOW_W);
    for i in 0..floor.0.len() {
        for k in (floor.0[0].len() / 2)..floor.0[0].len() {
            floor.0[i][k] = decrease_brightness(blue, floor.0[0].len() as u32 - k as u32);
        }
    }


    let mut player = Player::new();
    player.set_position(5.0, 3.0);

    // Main loop
    loop {

        // draw stupid ass skybox
        for x in 0..WINDOW_W {
                
            let screen_x = (x as f32 / WINDOW_W as f32 - 0.5) * VIEWPORT_SIZE;
            let ray_angle = player.view_angle + (screen_x / FOCAL_DISTANCE).atan();
                
           // Render skybox for upper half of screen
            for y in 0..(WINDOW_H / 2) {
                let vertical_ratio = y as f32 / (WINDOW_H / 2) as f32;
                let color = skybox.get_pixel(ray_angle, vertical_ratio);
                buffer.0[x][y] = color;
            }
        }

        // Add floor with goofy effect
        // Now just pulls from floor buffer2d to save time
        for i in 0..buffer.0.len() {
            for k in (buffer.0[0].len() / 2)..buffer.0[0].len() {
                buffer.0[i][k] = floor.0[i][k];
            }
        }

        // Render walls
        for c in 0..WINDOW_W {
            // Calculate ray angle for this column
            let screen_x = (c as f32 / WINDOW_W as f32 - 0.5) * VIEWPORT_SIZE;
            let ray_angle = player.view_angle + (screen_x / FOCAL_DISTANCE).atan();

            // Ray direction
            let dx = ray_angle.cos() / RAY_FINENESS;
            let dy = ray_angle.sin() / RAY_FINENESS;

            let mut ray_x = player.position.x;
            let mut ray_y = player.position.y;

            while ray_x <= map[0].len() as f32 - 1.0
                && ray_x >= 0.0
                && ray_y <= map.len() as f32 - 1.0
                && ray_y >= 0.0
            {
                let (ray_x_floor, ray_y_floor) = (ray_x.floor(), ray_y.floor());

                if map[ray_y_floor as usize][ray_x_floor as usize] == 1 {
                    let distance = ((ray_x - player.position.x).powf(2.0)
                        + (ray_y - player.position.y).powf(2.0))
                    .sqrt();
                    // Determine the proper u for the texturing, the way i'm doing this is a little jank
                    // but whatever #proof of concept
                    let u = || {
                        let ray_x_u = ray_x - ray_x_floor;
                        let ray_y_u = ray_y - ray_y_floor;

                        if ray_x_u < 1.0 / RAY_FINENESS || ray_x_u > (1.0 - 1.0 / RAY_FINENESS) {
                            return ray_y_u;
                        }

                        ray_x_u
                    };
                    
                    draw_line_textured(&mut buffer, c, &wall_texture, u(), distance, screen_x);
                    break;
                }

                ray_x += dx;
                ray_y += dy;
            }
        }

        buffer.to_screen(&mut screen_buffer);
        window
            .update_with_buffer(&screen_buffer, WINDOW_W, WINDOW_H)
            .expect("Window failed to update");
        buffer.flush();

        // cant believe this works, adding input checks

        if window.is_key_down(Key::Right) {
            player.update_angle(LOOK_SENSE);
        }

        if window.is_key_down(Key::Left) {
            player.update_angle(-1.0 * LOOK_SENSE);
        }

        // Add all movements together THEN apply
        let mut nx = 0.0;
        let mut ny = 0.0;

        if window.is_key_down(Key::W) {
            nx += player.view_angle.cos() * PLAYER_VELOCITY;
            ny += player.view_angle.sin() * PLAYER_VELOCITY;
        }

        if window.is_key_down(Key::S) {
            nx += -1.0 * player.view_angle.cos() * PLAYER_VELOCITY;
            ny += -1.0 * player.view_angle.sin() * PLAYER_VELOCITY;
        }

        if window.is_key_down(Key::A) {
            nx += player.view_angle.sin() * PLAYER_VELOCITY;
            ny += -1.0 * player.view_angle.cos() * PLAYER_VELOCITY;
        }

        if window.is_key_down(Key::D) {
            nx += -1.0 * player.view_angle.sin() * PLAYER_VELOCITY;
            ny += player.view_angle.cos() * PLAYER_VELOCITY;
        }

        player.update_position_checked(nx, ny, &map);
    }
}

#[inline(always)]
fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn decrease_brightness(color: u32, amount: u32) -> u32 {
    let mut r = color >> 16;
    let mut g = (color >> 8) & 255;
    let mut b = color & 255;

    if amount <= r {
        r -= amount;
    } else {
        r = 0;
    }
    if amount <= g {
        g -= amount;
    } else {
        g = 0;
    }
    if amount <= b {
        b -= amount;
    } else {
        b = 0;
    }

    (r << 16) | (g << 8) | b
}

fn increase_brightness(color: u32, amount: u32) -> u32 {
    let mut r = color >> 16;
    let mut g = (color >> 8) & 255;
    let mut b = color & 255;

    if r + amount <= 255 {
        r += amount;
    } else {
        r = 255;
    }
    if g + amount <= 255 {
        g += amount;
    } else {
        g = 255;
    }
    if b + amount <= 255 {
        b += amount;
    } else {
        b = 255;
    }

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

/// Includes light calculation
/// Also, the large number of seemingly arbitrary parameters are passed to stop
/// recalculations, the inline should stop arg passing from being a bottleneck
/// definetely a sign to refactor later
#[inline(always)]
fn draw_line_textured(buffer: &mut Buffer2D, c: usize, texture: &Texture, u: f32, distance: f32, screen_x: f32) {
    let corrected_distance = distance * (screen_x / FOCAL_DISTANCE as f32).cos();
    let h = (WINDOW_H as f32 / (corrected_distance + HEIGHT_ADJUSTMENT)) as u32;
    
    let h_bounded = h.min(WINDOW_H as u32);
    let offset = (WINDOW_H - h_bounded as usize) / 2;
    let mut color: u32 = 0;

    // Going to step for every v for each pixel being drawn
    let v_step: f32 = 1.0 / h as f32;
    let mut v: f32 = 0.0;
    for i in offset..offset + h_bounded as usize {
        color = texture.get_pixel_uv(u, v);
        buffer.0[c][i] = decrease_brightness(color, ((distance + 2.0) * (distance + 2.0) * SHADOW_ADJUSTMENT) as u32);
        v += v_step;
    }
}
