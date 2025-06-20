use std::error::Error;
use std::fmt;

use super::{Position, Camera};

#[derive(Debug)]
struct CameraBuildError {
    reason: &'static str,
}

// Majorly un-needed now
impl Error for CameraBuildError {}

impl fmt::Display for CameraBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = self.reason;
        write!(f, "{reason}")
    }
}
pub struct CameraOptions {
    position: Position,
    view_angle: f32,
    focal_distance: f32,
    viewport_size: f32,
    ray_fineness: f32,
    camera_fog: CameraFog,
}

impl Default for CameraOptions {
    fn default() -> Self {
        Self {
            position: Position{ x: 0.0, y: 0.0 },
            view_angle: 0.0,
            focal_distance: 1.0,
            viewport_size: 1.0,
            ray_fineness: 100.0,
            camera_fog: CameraFog::None,
        }
    }
}

impl Into<CameraOptionsBuilder> for CameraOptions {
    fn into(self) -> CameraOptionsBuilder {
        CameraOptionsBuilder {
            position: self.position,
            view_angle: self.view_angle,
            focal_distance: self.focal_distance,
            viewport_size: self.viewport_size,
            ray_fineness: self.ray_fineness,
            camera_fog: self.camera_fog,
        }
    }
}

impl Into<CameraOptions> for CameraOptionsBuilder {
    fn into(self) -> CameraOptions {
        CameraOptions {
            position: self.position,
            view_angle: self.view_angle,
            focal_distance: self.focal_distance,
            viewport_size: self.viewport_size,
            ray_fineness: self.ray_fineness,
            camera_fog: self.camera_fog,
        }
    }
}

impl Into<Camera> for CameraOptions {
    fn into(self) -> Camera {
        Camera {
            position: self.position,
            view_angle: self.view_angle,
            focal_distance: self.focal_distance,
            viewport_size: self.viewport_size,
            ray_fineness: self.ray_fineness,
            camera_fog: self.camera_fog,
        }
    }
}

pub struct CameraOptionsBuilder {
    position: Position,
    view_angle: f32,
    focal_distance: f32,
    viewport_size: f32,
    ray_fineness: f32,
    camera_fog: CameraFog,
}

impl CameraOptionsBuilder {
    pub fn new() -> Self {
        Self {
            position: Position{ x: 0.0, y: 0.0 },
            view_angle: 0.0,
            focal_distance: 1.0,
            viewport_size: 1.0,
            ray_fineness: 100.0,
            camera_fog: CameraFog::None,
        }
    }

    pub fn position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn view_angle(&mut self, view_angle: f32) {
        self.view_angle = view_angle;
    }

    pub fn focal_distance(&mut self, focal_distance: f32) {
        self.focal_distance = focal_distance;
    }

    pub fn viewport_size(&mut self, viewport_size: f32) {
        self.viewport_size = viewport_size;
    }

    pub fn ray_fineness(&mut self, ray_fineness: f32) {
        self.ray_fineness = ray_fineness;
    }

    pub fn camera_fog(&mut self, camera_fog: CameraFog) {
        self.camera_fog = camera_fog;
    }

}

pub enum CameraFog {
    None,
    VisibleDistance(f32),
}