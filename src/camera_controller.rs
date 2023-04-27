pub(crate) struct CameraController {
    pub(crate) aim_sensitivity: f32,
    pub(crate) speed_factor: u16,
    pub(crate) speed: f32,
    pub(crate) yaw: f32,
    pub(crate) pitch: f32,
    pub(crate) mouse_pos_last_x: f64,
    pub(crate) mouse_pos_last_y: f64,
    pub(crate) min_fov_y: f32,
    pub(crate) max_fov_y: f32,
}
