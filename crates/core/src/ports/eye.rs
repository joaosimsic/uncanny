pub trait EyeController {
    fn look_at(&mut self, x: f32, y: f32);
    fn blink(&mut self);
    fn saccade(&mut self);
}
