pub struct CameraController {
    pub aim_sensitivity: f32,
    pub speed_factor: u16,
    pub speed: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub mouse_pos_last_x: f64,
    pub mouse_pos_last_y: f64,
    pub min_fov_y: f32,
    pub max_fov_y: f32,
}
