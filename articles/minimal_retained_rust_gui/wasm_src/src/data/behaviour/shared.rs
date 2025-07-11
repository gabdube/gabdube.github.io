use crate::shared::{PositionF32, pos};

pub fn move_to_with_speed(current: PositionF32, target: PositionF32, frame_delta: f32, base_speed: f32) -> PositionF32 {
    let angle = f32::atan2(target.y - current.y, target.x - current.x);
    let speed = base_speed * frame_delta;
    
    let cos_a = f32::cos(angle);
    let mut move_x = speed * cos_a;
    if cos_a > 0.0 {
        move_x = f32::min(move_x, target.x - current.x);
    } else if cos_a < 0.0 {
        move_x = f32::max(move_x, target.x - current.x);
    }

    let sin_a = f32::sin(angle);
    let mut move_y = speed * sin_a;
    if sin_a > 0.0 {
        move_y = f32::min(move_y, target.y - current.y);
    } else if sin_a < 0.0 {
        move_y = f32::max(move_y, target.y - current.y);
    }

    pos(current.x + move_x, current.y + move_y)
}
