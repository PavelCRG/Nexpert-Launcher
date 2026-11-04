#[cfg(test)]
mod tests {
    use super::Camera;

    #[test]
    fn camera_starts_looking_toward_the_world() {
        let camera = Camera::default();
        let direction = camera.forward();
        assert!(direction.z < -0.99);
        assert!(direction.x.abs() < 0.01);
    }

    #[test]
    fn camera_looks_up_and_down_with_pitch() {
        let mut camera = Camera::default();
        camera.look(0.0, -100.0);
        assert!(camera.forward().y > 0.0);
        camera.look(0.0, 200.0);
        assert!(camera.forward().y < 0.0);
    }

    #[test]
    fn moving_mouse_right_turns_camera_right() {
        let mut camera = Camera::default();
        camera.look(100.0, 0.0);
        assert!(camera.forward().x > 0.0);
    }
}

use glam::{Mat4, Vec3};

pub struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(8.0, 6.0, 21.0),
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.12,
        }
    }
}

impl Camera {
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn look(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.0025;
        self.pitch = (self.pitch - dy * 0.0025).clamp(-1.54, 1.54);
    }

    pub fn move_local(&mut self, forward: f32, right: f32, up: f32, delta: f32) {
        let direction = self.forward();
        let flat = Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero();
        let side = flat.cross(Vec3::Y).normalize_or_zero();
        self.position += (flat * forward + side * right + Vec3::Y * up) * (delta * 7.0);
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let view = Mat4::look_to_rh(self.position, self.forward(), Vec3::Y);
        Mat4::perspective_rh(70_f32.to_radians(), aspect, 0.1, 128.0) * view
    }
}
