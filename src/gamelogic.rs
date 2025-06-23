use crate::rendering::{Position, Canvas};
use minifb::Key;
pub trait Moveable {
    fn get_position(&self) -> Position;
    fn get_angle(&self) -> f32;
    fn set_position(&mut self, x: f32, y: f32);
    fn set_angle(&mut self, theta: f32);
    fn update_position(&mut self, x: f32, y: f32);
    fn update_angle(&mut self, theta: f32);
    fn update_position_checked(&mut self, dx: f32, dy: f32, map: &Vec<Vec<usize>>) {
        let Position { x, y} = self.get_position();
        let new_x = x + dx;
        let new_y = y + dy;

        if map[new_y.floor() as usize][new_x.floor() as usize] == 0 {
            self.set_position(new_x, new_y);
            return;
        }

        if map[new_y.floor() as usize][x.floor() as usize] == 0 {
            self.set_position(x, new_y);
        }

        if map[y.floor() as usize][new_x.floor() as usize] == 0 {
            self.set_position(new_x, y);
        }
    }
}

/// This is hilarious hope no one manages to drop a Movable before they drop
/// the movement controller
pub struct UserMovementController<'a> {
    pub entity: *mut dyn Moveable,
    pub move_speed: f32,
    pub look_sense: f32,
    pub _marker: std::marker::PhantomData<&'a mut dyn Moveable>
}

impl<'a> UserMovementController<'a> {

    pub fn new(entity: *mut dyn Moveable, move_speed: f32, look_sense: f32) -> Self {
        Self {
            entity,
            move_speed,
            look_sense,
            _marker: std::marker::PhantomData,
        }
    }

    /// Reads movement inputs and enforces bounds checking
    /// For a supplied map
    pub fn physics_input(&self, canvas: &Canvas, map: &Vec<Vec<usize>>) {
        
        if canvas.is_key_down(Key::Right) {
            unsafe { (*self.entity).update_angle(self.look_sense); }
        }

        if canvas.is_key_down(Key::Left) {
            unsafe { (*self.entity).update_angle(-1.0 * self.look_sense); }
        }
        
        let mut nx = 0.0;
        let mut ny = 0.0;

        let angle = unsafe { (*self.entity).get_angle() }; // The lion does not use Rc<RefCell>

        if canvas.is_key_down(Key::W) {
            nx += angle.cos() * self.move_speed;
            ny += angle.sin() * self.move_speed;
        }

        if canvas.is_key_down(Key::S) {
            nx += -1.0 * angle.cos() * self.move_speed;
            ny += -1.0 * angle.sin() * self.move_speed;
        }

        if canvas.is_key_down(Key::A) {
            nx += angle.sin() * self.move_speed;
            ny += -1.0 * angle.cos() * self.move_speed;
        }

        if canvas.is_key_down(Key::D) {
            nx += -1.0 * angle.sin() * self.move_speed;
            ny += angle.cos() * self.move_speed;
        }

        unsafe { (*self.entity).update_position_checked(nx, ny, map); }
    }
}