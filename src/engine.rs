use minifb::{Window, WindowOptions, Key};
use image::{GenericImageView, DynamicImage};

struct Position {
    x: f32,
    y: f32, // this is a 2d x,y coordinate plane
}

pub struct Camera {
    position: Position,
    view_angle: f32, // Principal axis is facing right, deviation is in radians.
    focal_distance: f32,
    viewport_size: f32,
}

impl Camera {
    fn new(fd: f32, vs: f32) -> Self {
        Self {
            position: Position { x: 0.0, y: 0.0 },
            view_angle: 0.0,
            focal_distance: fd,
            viewport_size: vs,
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