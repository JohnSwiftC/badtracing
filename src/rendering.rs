use minifb::{Window, WindowOptions, Key};
use image::{GenericImageView, DynamicImage};

pub struct Canvas {
    window: Window,
    buffer: Buffer2D,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    pub fn new(name: &'static str, width: usize, height: usize) -> Result<Self, minifb::Error> {
        Ok(
            Self {
                window: Window::new(name, width, height, WindowOptions::default())?,
                buffer: Buffer2D::new(height, width),
                width,
                height,
            }
        )
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
    fn new(fd: f32, vs: f32, rf: f32) -> Self {
        Self {
            position: Position { x: 0.0, y: 0.0 },
            view_angle: 0.0,
            focal_distance: fd,
            viewport_size: vs,
            ray_fineness: rf,
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

    fn raycast_map(canvas: &mut Canvas, map: &Vec<Vec<usize>>, textures: &[]) {
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

        if let TextureOption::Color(c) = self.image {
            return c;
        }

        let x = (u * self.width as f32).floor() as u32;
        let y = (v * self.height as f32).floor() as u32;

        // Also, I know that I only need to draw columns so this can be heavily optimized but just poc for now

        let pixel = self.image.get_pixel(x, y);

        from_u8_rgb(pixel[0], pixel[1], pixel[2])

    }
}

pub struct Buffer2D(Vec<Vec<u32>>);

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