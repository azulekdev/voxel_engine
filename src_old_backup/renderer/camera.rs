use glam::{Mat4, Vec3};
// use glfw::{Action, Key};

pub struct Camera {
    pub position: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub world_up: Vec3,

    pub yaw: f32,
    pub pitch: f32,

    #[allow(dead_code)]
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    #[allow(dead_code)]
    pub zoom: f32,
}

impl Camera {
    pub fn new(position: Vec3, up: Vec3, yaw: f32, pitch: f32) -> Self {
        let mut camera = Camera {
            position,
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::ZERO,
            right: Vec3::ZERO,
            world_up: up,
            yaw,
            pitch,
            movement_speed: 10.0,
            mouse_sensitivity: 0.1,
            zoom: 45.0,
        };
        camera.update_camera_vectors();
        camera
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.front, self.up)
    }

    pub fn process_mouse_movement(
        &mut self,
        mut xoffset: f32,
        mut yoffset: f32,
        constrain_pitch: bool,
    ) {
        xoffset *= self.mouse_sensitivity;
        yoffset *= self.mouse_sensitivity;

        self.yaw += xoffset;
        self.pitch += yoffset;

        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        self.update_camera_vectors();
    }

    fn update_camera_vectors(&mut self) {
        let mut front = Vec3::ZERO;
        front.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        front.y = self.pitch.to_radians().sin();
        front.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.front = front.normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
}
