use uncanny_core::ports::EyeController;

pub struct StubEye;

impl EyeController for StubEye {
    fn look_at(&mut self, x: f32, y: f32) {
        eprintln!("[eye] look_at({x:.2}, {y:.2})");
    }
    fn blink(&mut self) {
        eprintln!("[eye] blink");
    }
    fn saccade(&mut self) {
        eprintln!("[eye] saccade");
    }
}
