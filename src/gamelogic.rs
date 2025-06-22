use crate::rendering::{Position};
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