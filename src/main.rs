#![allow(dead_code)]

use minifb::Key;

mod rendering;

use rendering::Camera;
use rendering::cameraspec;
use rendering::cameraspec::{CameraFog, CameraOptions, CameraOptionsBuilder};

const WINDOW_W: usize = 700;
const WINDOW_H: usize = 700;
const FPS: usize = 60;
const FOCAL_DISTANCE: f32 = WINDOW_H as f32 / WINDOW_W as f32;
const VIEWPORT_SIZE: f32 = 1.0; // Width of the viewport used for calculations
const RAY_FINENESS: f32 = 100.0; // How much the dx and dy are divided by for each step in the raycast. Higher values lead to more accurate casts but slower performance
const PLAYER_VELOCITY: f32 = 0.04; // Scales the movement amount determined by the sin and cosine
const LOOK_SENSE: f32 = 0.02; // Speed of rotation with arrow keys

fn main() {
    let map: Vec<Vec<usize>> = vec![
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1],
        vec![1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1],
        vec![1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
        vec![1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];

    // While the manually implemented functions on camera that act on different
    // structs isnt that nice to use (I did think about using a trait and function
    // That takes in the camera context as a way to get around this) I don't want to
    // force even more data to be moved around and passed to functions every frame
    // also the engine is bad, its in the name.

    let skybox = rendering::Skybox::load_from_file("skybox.jpg").expect("skybox failed to load");
    let wall_texture =
        rendering::Texture::load_from_file("wall.jpg").expect("wall texture failed to load");
    let floor_color = from_u8_rgb(0, 0, 255);

    let mut canvas = rendering::Canvas::new("badtracing", WINDOW_W, WINDOW_H).unwrap();
    let camera_options: CameraOptions = cameraspec::CameraOptionsBuilder::new()
                                .camera_fog(CameraFog::VisibleDistance { fog_dist: 2.0, fog_color: from_u8_rgb(50, 50, 50) })
                                .into();
    let mut camera: Camera = camera_options.into();

    camera.set_position(4.0, 4.0);
    canvas.set_target_fps(FPS);
    // Main loop
    loop {
        camera.draw_simple_floor(&mut canvas, floor_color);
        camera.draw_skybox(&mut canvas, &skybox);
        camera.raycast_map(&mut canvas, &map, &[&wall_texture]);
        canvas.update();

        if canvas.is_key_down(Key::Right) {
            camera.update_angle(LOOK_SENSE);
        }

        if canvas.is_key_down(Key::Left) {
            camera.update_angle(-1.0 * LOOK_SENSE);
        }

        // Add all movements together THEN apply
        let mut nx = 0.0;
        let mut ny = 0.0;

        if canvas.is_key_down(Key::W) {
            nx += camera.view_angle.cos() * PLAYER_VELOCITY;
            ny += camera.view_angle.sin() * PLAYER_VELOCITY;
        }

        if canvas.is_key_down(Key::S) {
            nx += -1.0 * camera.view_angle.cos() * PLAYER_VELOCITY;
            ny += -1.0 * camera.view_angle.sin() * PLAYER_VELOCITY;
        }

        if canvas.is_key_down(Key::A) {
            nx += camera.view_angle.sin() * PLAYER_VELOCITY;
            ny += -1.0 * camera.view_angle.cos() * PLAYER_VELOCITY;
        }

        if canvas.is_key_down(Key::D) {
            nx += -1.0 * camera.view_angle.sin() * PLAYER_VELOCITY;
            ny += camera.view_angle.cos() * PLAYER_VELOCITY;
        }

        camera.update_position_checked(nx, ny, &map);
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
