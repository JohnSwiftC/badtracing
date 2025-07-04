#![allow(dead_code)]

pub mod cameraspec;

use image::{DynamicImage, GenericImageView};
use minifb::{Key, Window, WindowOptions};
use std::{path::Path};

use crate::gamelogic::Moveable;

const RAY_FINENESS: f32 = 100.0;

pub struct Canvas {
    window: Window,
    buffer: Buffer2D,
    pub width: usize,
    pub height: usize,
    screen_buffer: Vec<u32>,
    depth_buffer: Vec<f32>,
}

impl Canvas {
    pub fn new(name: &'static str, width: usize, height: usize) -> Result<Self, minifb::Error> {
        Ok(Self {
            window: Window::new(name, width, height, WindowOptions::default())?,
            buffer: Buffer2D::new(height, width),
            width,
            height,
            screen_buffer: vec![0; width * height],
            depth_buffer: vec![std::f32::MAX; width], // Depth for each column on the canvas
        })
    }

    pub fn update(&mut self) {
        self.buffer.to_screen(&mut self.screen_buffer);
        let _ = self
            .window
            .update_with_buffer(&self.screen_buffer, self.width, self.height);
        self.buffer.flush();
        self.flush_depth();
    }

    /// Resets the depth buffer for every column to be at max depth
    fn flush_depth(&mut self) {
        for c in &mut self.depth_buffer {
            *c = std::f32::MAX;
        }
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }

    pub fn set_target_fps(&mut self, fps: usize) {
        self.window.set_target_fps(fps);
    }
}

pub struct Skybox {
    image: DynamicImage,
    width: u32,
    height: u32,
}

impl Skybox {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();

        Ok(Skybox {
            image,
            width,
            height,
        })
    }

    // Get pixel color from skybox based on viewing angle and vertical position
    pub fn get_pixel(&self, angle: f32, vertical_ratio: f32) -> u32 {
        let two_pi = 2.0 * std::f32::consts::PI;
        let normalized_angle = (angle % (two_pi) + two_pi) / two_pi;
        let u = (normalized_angle * self.width as f32) as u32 % self.width;

        let mut v = ((1.0 - vertical_ratio.clamp(0.0, 1.0)) * self.height as f32) as u32;
        v = v.min(self.height - 1);

        let pixel = self.image.get_pixel(u, v);

        from_u8_rgb(pixel[0], pixel[1], pixel[2])
    }
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32, // this is a 2d x,y coordinate plane
}

pub struct Camera {
    position: Position,
    pub view_angle: f32, // Principal axis is facing right, deviation is in radians.
    focal_distance: f32,
    viewport_size: f32,
    ray_fineness: f32,
    camera_fog: cameraspec::CameraFog,
}

impl Moveable for Camera {
    fn get_position(&self) -> Position {
        self.position
    }
    fn get_angle(&self) -> f32 {
        self.view_angle
    }
    fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }
    fn set_angle(&mut self, theta: f32) {
        if theta < 0.0 {
            self.view_angle = 2.0 * std::f32::consts::PI + (theta % (2.0 * std::f32::consts::PI));
            return;
        }
        self.view_angle = theta % (2.0 * std::f32::consts::PI);
    }
    fn update_position(&mut self, x: f32, y: f32) {
        self.position.x += x;
        self.position.y += y;
    }
    fn update_angle(&mut self, theta: f32) {
        if theta < 0.0 {
            self.view_angle += theta;
            if self.view_angle < 0.0 {
                self.view_angle += 2.0 * std::f32::consts::PI;
            }
            self.view_angle %= 2.0 * std::f32::consts::PI;
            return;
        }
        self.view_angle = (theta + self.view_angle) % (2.0 * std::f32::consts::PI);
    }
}

impl Camera {
    pub fn new(fd: f32, vs: f32, rf: f32) -> Self {
        Self {
            position: Position { x: 0.0, y: 0.0 },
            view_angle: 0.0,
            focal_distance: fd,
            viewport_size: vs,
            ray_fineness: rf,
            camera_fog: cameraspec::CameraFog::None,
        }
    }

    pub fn draw_skybox(&mut self, canvas: &mut Canvas, skybox: &Skybox) {
        for x in 0..canvas.width {
            let screen_x = (x as f32 / canvas.width as f32 - 0.5) * self.viewport_size;
            let ray_angle = self.view_angle + (screen_x / self.focal_distance).atan();

            for y in 0..(canvas.height / 2) {
                let vertical_ratio = y as f32 / (canvas.height / 2) as f32;
                let color = skybox.get_pixel(ray_angle, vertical_ratio);
                canvas.buffer.0[x][y] = color;
            }
        }
    }

    pub fn draw_simple_floor(&mut self, canvas: &mut Canvas, color: u32) {
        for i in 0..canvas.width {
            for k in (canvas.height / 2)..canvas.height {
                canvas.buffer.0[i][k] = decrease_brightness(color, canvas.height as u32 - k as u32);
            }
        }
    }

    /// This function is the main rendering function of the camera. Renders the map, draws fog optionally
    /// must be used or changed for things that interact with the map, ie sprites or fog
    /// (fog being rendered depends on whether or not it is broken by a piece of wall)
    pub fn main(&self, canvas: &mut Canvas, map: &Vec<Vec<usize>>, textures: &[&Texture]) {
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

                // Various height and distance values
                // TODO: This now runs every single ray step = SLOW, perform some optimizations
                let distance = ((ray_x - self.position.x).powf(2.0)
                    + (ray_y - self.position.y).powf(2.0))
                .sqrt();
                let corrected_distance = distance * (screen_x / self.focal_distance as f32).cos();

                let h = (canvas.height as f32 / corrected_distance) as u32;
                let h_bounded = h.min(canvas.height as u32);
                let offset = (canvas.height - h_bounded as usize) / 2;

                use cameraspec::CameraFog;
                match self.camera_fog {
                    CameraFog::VisibleDistance {
                        fog_dist,
                        fog_color,
                    } => {
                        // Note that these are the scoped values from the enum
                        if distance > fog_dist {
                            for i in 0..offset + h_bounded as usize {
                                canvas.buffer.0[c][i] = fog_color;
                            }

                            break; // Skip rest of rendering for this column, fog covers it
                        }
                    }
                    CameraFog::None => (),
                }

                if map[ray_y_floor as usize][ray_x_floor as usize] != 0 {
                    // Lets quickly see if we should draw this
                    if canvas.depth_buffer[c] < corrected_distance {
                        break;
                    }

                    canvas.depth_buffer[c] = corrected_distance;

                    // Texturing, u and v values found and used
                    let mut color: u32;

                    let u = (|| {
                        let ray_x_u = ray_x - ray_x_floor;
                        let ray_y_u = ray_y - ray_y_floor;

                        if ray_x_u < 1.0 / RAY_FINENESS || ray_x_u > (1.0 - 1.0 / RAY_FINENESS) {
                            return ray_y_u;
                        }

                        ray_x_u
                    })();

                    // Going to step for every v for each pixel being drawn
                    let v_step: f32 = 1.0 / h as f32;
                    let mut v: f32 = 0.0;

                    // Check to see if player is too close to see the very top
                    // Finds the proper initial v value if the top of the texture
                    // is off screen.
                    if h > h_bounded {
                        let d = (h - h_bounded) / 2;
                        v = d as f32 / h as f32;
                    }

                    // If fog is being rendered, we also want it to appear above the block
                    // This will make a skybox useless, so might add some sort of transparency as we go up
                    if let CameraFog::VisibleDistance { fog_color, .. } = &self.camera_fog {
                        for i in 0..offset {
                            canvas.buffer.0[c][i] = *fog_color; // Testing additive colors in fog for the skybox
                        }
                    }

                    for i in offset..offset + h_bounded as usize {
                        color = textures[map[ray_y_floor as usize][ray_x_floor as usize] - 1]
                            .get_pixel_uv(u, v);
                        canvas.buffer.0[c][i] = decrease_brightness(
                            color,
                            ((distance + 2.0) * (distance + 2.0) * 2.5) as u32,
                        ); // 2.5 is the shadow adjustment
                        v += v_step;
                    }
                    break;
                }

                ray_x += dx;
                ray_y += dy;
            }
        }
    }

    pub fn render_sprites(&self, canvas: &mut Canvas, sprites: &[&Sprite]) {
        for &s in sprites {
            let x = s.position.x - self.position.x;
            let y = s.position.y - self.position.y;

            // For some reason the view angle starts with a pi/2 offset relative to the
            // angle we want here. This gives an angle I can easily use to compare with the sprite angle
            let adjusted_va =
                (self.view_angle + std::f32::consts::FRAC_PI_2 + std::f32::consts::PI)
                    % (2.0 * std::f32::consts::PI);

            let mut sprite_angle = (x.abs() / y.abs()).atan();
            // Convert the angle given into [0, 2pi]
            if y < 0.0 && x > 0.0 {
                sprite_angle = std::f32::consts::PI - sprite_angle;
            } else if y > 0.0 && x < 0.0 {
                sprite_angle = (2.0 * std::f32::consts::PI) - sprite_angle;
            } else if y < 0.0 && x < 0.0 {
                sprite_angle += std::f32::consts::PI;
            }

            // For reasons I don't feel like looking into, the angle im getting here
            // is rotating counter clockewise from a birds eye view. This swaps the direction.
            sprite_angle = (2.0 * std::f32::consts::PI) - sprite_angle;

            //println!("{} : {}", sprite_angle, adjusted_va);

            let (left_bound, right_bound, sector_angle) = {
                // This gets the angle between the camera border and the center focal line
                // Then used to find the angles of the two camera borders globally
                let deviation = (self.viewport_size / 2.0 / self.focal_distance).atan();

                let mut left_bound = adjusted_va - deviation;
                if left_bound < 0.0 {
                    left_bound = (2.0 * std::f32::consts::PI) + left_bound;
                }

                let right_bound = (adjusted_va + deviation) % (2.0 * std::f32::consts::PI);

                (left_bound, right_bound, deviation * 2.0)
            };

            // Don't draw a sprite if its not in the visible angle of the camera
            let c = match is_in_sector(left_bound, right_bound, sprite_angle) {
                Ok(a) => {
                    let v = a / sector_angle;
                    (v * canvas.width as f32) as usize
                }
                Err(()) => continue,
            };

            let screen_x = (c as f32 / canvas.width as f32 - 0.5) * self.viewport_size;
            let distance = (x * x + y * y).sqrt();
            let corrected_distance = distance * (screen_x / self.focal_distance as f32).cos();
            let h = (canvas.height as f32 / corrected_distance) as u32;

            let h_bounded = h.min(canvas.height as u32);
            let mut offset = (canvas.height - h_bounded as usize) / 2;

            // Apply scaling magics
            let scaled_h = (h_bounded as f32 * s.scale) as u32;
            offset += (h_bounded - scaled_h) as usize;
            let h_bounded = scaled_h; // Quick reassign

            // Get a correct width
            let aspect_ratio = s.texture.width as f32 / s.texture.height as f32;
            let corrected_width = (aspect_ratio * h_bounded as f32).floor() as usize;

            // c is the column to draw the sprite on

            let mut v = 0.0;
            let v_step = 1.0 / h_bounded as f32;

            let mut u;
            let u_step = 1.0 / corrected_width as f32;
            let u_bounded: f32;

            // Bounding to 0 isnt really something to do here because of the unsigned type
            let mut left = 0;
            if corrected_width / 2 <= c {
                left = c - corrected_width / 2;
                u_bounded = 0.0;
            } else {
                // Adjust u
                u_bounded = ((corrected_width / 2) - c) as f32 * u_step;
            }

            u = u_bounded;

            let right = c + corrected_width / 2;
            let right_bounded = right.min(canvas.width);

            // Each i, k pair should be a pixel being drawn for the sprite
            for i in offset..offset + h_bounded as usize {
                for k in left..right_bounded {
                    if canvas.depth_buffer[k] < corrected_distance {
                        u += u_step; // Make sure we still texture correctly
                        continue; // This will check depth for every column,
                        // Should rework loop to do this column by column instead of the
                        // current row by row to stop repeat checks on the columns
                        // TODO
                    } else {
                        canvas.depth_buffer[k] = corrected_distance;
                    }
                    canvas.buffer.0[k][i] = s.texture.get_pixel_uv(u, v);
                    u += u_step;
                }
                u = u_bounded;
                v += v_step;
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

pub struct Sprite<'a> {
    position: Position,
    texture: &'a Texture,
    scale: f32,
}

impl<'a> Sprite<'a> {
    pub fn from_texture(texture: &'a Texture) -> Self {
        Self {
            position: Position { x: 0.0, y: 0.0 },
            texture: texture,
            scale: 1.0,
        }
    }

    pub fn scale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

impl Moveable for Sprite<'_> {
    fn get_position(&self) -> Position {
        self.position
    }

    fn get_angle(&self) -> f32 {
        0.0 // Redundant for sprites, atleast for now
    }

    fn set_angle(&mut self, _theta: f32) {
        // None
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    fn update_angle(&mut self, _theta: f32) {
        // None
    }

    fn update_position(&mut self, x: f32, y: f32) {
        self.position.x += x;
        self.position.y += y;
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

/// Returns true if an angle is within another two on the unit circle
/// This function is intended for angles that are locked to [0, 2pi],
/// thus the logic checks
///
/// (Remember, a left bound could technically have a value greater than that of
/// the right bound if the range straddles the positive y axis.)
#[inline(always)]
fn is_in_sector(left_bound: f32, right_bound: f32, angle: f32) -> Result<f32, ()> {
    if left_bound < right_bound && angle > left_bound && angle < right_bound {
        return Ok(angle - left_bound);
    } else if left_bound > right_bound && angle > left_bound && angle > right_bound {
        return Ok(angle - left_bound);
    } else if left_bound > right_bound && angle < left_bound && angle < right_bound {
        return Ok((2.0 * std::f32::consts::PI) - left_bound + angle);
    }

    Err(())
}
