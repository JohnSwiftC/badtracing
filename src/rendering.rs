use minifb::{Window, WindowOptions, Key};
use image::{GenericImageView, DynamicImage};
use std::path::Path;

const RAY_FINENESS: f32 = 100.0;

pub struct Canvas {
    window: Window,
    buffer: Buffer2D,
    pub width: usize,
    pub height: usize,
    screen_buffer: Vec<u32>,
}

impl Canvas {
    pub fn new(name: &'static str, width: usize, height: usize) -> Result<Self, minifb::Error> {
        Ok(
            Self {
                window: Window::new(name, width, height, WindowOptions::default())?,
                buffer: Buffer2D::new(height, width),
                width,
                height,
                screen_buffer: vec![0; width * height],
            }
        )
    }

    pub fn update(&mut self) {
        self.buffer.to_screen(&mut self.screen_buffer);
        self.window.update_with_buffer(&self.screen_buffer, self.width, self.height);
        self.buffer.flush();
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }
}

struct Position {
    x: f32,
    y: f32, // this is a 2d x,y coordinate plane
}

pub struct Camera {
    position: Position,
    view_angle: f32, // Principal axis is facing right, deviation is in radians.
    focal_distance: f32,
    viewport_size: f32,
    ray_fineness: f32,
}

impl Camera {
    pub fn new(fd: f32, vs: f32, rf: f32) -> Self {
        Self {
            position: Position { x: 0.0, y: 0.0 },
            view_angle: 0.0,
            focal_distance: fd,
            viewport_size: vs,
            ray_fineness: rf,
        }
    }

    /// Updates absolute position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    /// Updates relative position
    pub fn update_position(&mut self, x: f32, y: f32) {
        self.position.x += x;
        self.position.y += y;
    }

    /// Updates relative position with collision detection
    pub fn update_position_checked(&mut self, x: f32, y: f32, map: &Vec<Vec<usize>>) {
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
    pub fn set_angle(&mut self, theta: f32) {
        self.view_angle = theta;
    }

    /// Updates relative angle
    pub fn update_angle(&mut self, theta: f32) {
        self.view_angle += theta;
    }

    pub fn raycast_map(&self, canvas: &mut Canvas, map: &Vec<Vec<usize>>, textures: &[&Texture]) {
        for c in 0..canvas.width {
            // Calculate ray angle for this column
            let screen_x = (c as f32 / canvas.width as f32 - 0.5) * self.viewport_size;
            let ray_angle = self.view_angle + (screen_x / self.focal_distance).atan();

            // Ray direction
            let dx = ray_angle.cos() / self.ray_fineness;
            let dy = ray_angle.sin() / self.ray_fineness;

            let mut ray_x = self.position.x;
            let mut ray_y = self.position.y;

            while ray_x <= map[0].len() as f32 - 1.0
                && ray_x >= 0.0
                && ray_y <= map.len() as f32 - 1.0
                && ray_y >= 0.0
            {
                let (ray_x_floor, ray_y_floor) = (ray_x.floor(), ray_y.floor());

                if map[ray_y_floor as usize][ray_x_floor as usize] != 0 {
                    let distance = ((ray_x - self.position.x).powf(2.0)
                        + (ray_y - self.position.y).powf(2.0))
                    .sqrt();
                    // Determine the proper u for the texturing, the way i'm doing this is a little jank
                    // but whatever #proof of concept
                    let u = (|| {
                        let ray_x_u = ray_x - ray_x_floor;
                        let ray_y_u = ray_y - ray_y_floor;

                        if ray_x_u < 1.0 / RAY_FINENESS || ray_x_u > (1.0 - 1.0 / RAY_FINENESS) {
                            return ray_y_u;
                        }

                        ray_x_u
                    })();
                    
                    let corrected_distance = distance * (screen_x / self.focal_distance as f32).cos();
                    let h = (canvas.height as f32 / corrected_distance) as u32;
                    
                    let h_bounded = h.min(canvas.height as u32);
                    let offset = (canvas.height - h_bounded as usize) / 2;
                    let mut color: u32 = 0;

                    // Going to step for every v for each pixel being drawn
                    let v_step: f32 = 1.0 / h as f32;
                    let mut v: f32 = 0.0;
                    for i in offset..offset + h_bounded as usize {
                        color = textures[map[ray_y_floor as usize][ray_x_floor as usize] - 1].get_pixel_uv(u, v);
                        canvas.buffer.0[c][i] = decrease_brightness(color, ((distance + 2.0) * (distance + 2.0) * 5.0) as u32); // 5.0 is the shadow adjustment
                        v += v_step;
                    }
                    break;
                }

                ray_x += dx;
                ray_y += dy;
            }
        }
    }
}

enum TextureOption {
    Image(DynamicImage),
    Color(u32),
}

pub struct Texture {
    image: TextureOption,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        
        Ok(Self {
            image: TextureOption::Image(image),
            width,
            height,
        })
    }

    pub fn from_color(color: u32) -> Self {
        Self {
            image: TextureOption::Color(color),
            width: 1,
            height: 1,
        }
    }

    fn get_pixel_uv(&self, u: f32, v: f32) -> u32 {
        // U is relative to x, v is relative to y here
        // I'm using uv because a size of a wall is 1, so we can easily calculate uv with a ray position and wall corner position

        if let TextureOption::Color(c) = &self.image {
            return *c;
        }

        let x = (u * self.width as f32).floor() as u32;
        let y = (v * self.height as f32).floor() as u32;

        // Also, I know that I only need to draw columns so this can be heavily optimized but just poc for now
        if let TextureOption::Image(i) = &self.image {
            let pixel = i.get_pixel(x, y);
            return from_u8_rgb(pixel[0], pixel[1], pixel[2]);
        }

        0
    }
}

pub struct Buffer2D(Vec<Vec<u32>>);

impl Buffer2D {
    fn new(height: usize, width: usize) -> Self {
        Self(vec![vec![0; height]; width])
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
                self.0[i][k] = 0;
            }
        }
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