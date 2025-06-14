use minifb::{Window, WindowOptions, Key};
use image::{GenericImageView, DynamicImage};
use std::path::Path;

#[inline(always)]
fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

struct Buffer2D(Vec<Vec<u32>>);

impl Buffer2D {
    fn new(width: usize, height: usize) -> Self {
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

struct Position {
    x: f32,
    y: f32, // this is a 2d x,y coordinate plane
}

// Trait used to write things to be rendered to the screen
trait Renderable {
    fn render(&self, buffer: &mut Buffer2D);
}

struct GameMonolith<'a> {
    window: Window,
    screen_buffer: Buffer2D,
    player: Player,
    map: Option<Vec<Vec<MapObject<'a>>>>,
    pre_render: Vec<Box<dyn Renderable>>, // Renders in layers before the camera renders the world, in order
    post_render: Vec<Box<dyn Renderable>>, // Renders in layers after the camera renders the world, in order
}

impl<'a> GameMonolith<'a> {
    pub fn create_with_dimensions(name: &str, width: usize, height: usize) -> Result<Self, minifb::Error> {
        Ok(Self {
            window: Window::new(name, width, height, WindowOptions::default())?,
            screen_buffer: Buffer2D::new(width, height),
            player: Player::new(),
            map: None,
            pre_render: Vec::new(),
            post_render: Vec::new(),
        })
    }
}

struct MapObject<'a> {
    texture: &'a Texture,
}

enum TextureOption {
    Image(DynamicImage),
    Color(u32),
}

struct Texture {
    image: TextureOption,
    width: u32,
    height: u32,
}

impl Texture {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        
        Ok(Self {
            image: TextureOption::Image(image),
            width,
            height,
        })
    }

    fn create_with_color(color: u32) -> Self {
        Self {
            image: TextureOption::Color(color),
            width: 1,
            height: 1,
        }
    }

    fn get_pixel_uv(&self, u: f32, v: f32) -> u32 {
        match &self.image {
            TextureOption::Image(dyn_image) => {
                let x = (u * self.width as f32).floor() as u32;
                let y = (v * self.height as f32).floor() as u32;
                let pixel = dyn_image.get_pixel(x, y);
                return from_u8_rgb(pixel[0], pixel[1], pixel[2]);
            },
            TextureOption::Color(color) => {return *color;}
        }
    }
}

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