use macroquad::prelude::*;

pub const REFERENCE_DISTANCE: f32 = 1000.0;
const MIN_DISTANCE: f32 = REFERENCE_DISTANCE / 30.0;
const MAX_DISTANCE: f32 = REFERENCE_DISTANCE / 0.05;

const DEFAULT_YAW: f32 = -1.1;
const DEFAULT_PITCH: f32 = 0.55;
const DEFAULT_DISTANCE: f32 = 1500.0;

pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,

    goal_yaw: f32,
    goal_pitch: f32,
    goal_distance: f32,
    goal_target: Vec3,

    pub fovy: f32,
    pub auto_rotate: bool,
}

impl OrbitCamera {
    pub fn new() -> Self {
        OrbitCamera {
            yaw: DEFAULT_YAW,
            pitch: DEFAULT_PITCH,
            distance: DEFAULT_DISTANCE * 2.2,
            target: Vec3::ZERO,
            goal_yaw: DEFAULT_YAW,
            goal_pitch: DEFAULT_PITCH,
            goal_distance: DEFAULT_DISTANCE,
            goal_target: Vec3::ZERO,
            fovy: 50.0_f32.to_radians(),
            auto_rotate: false,
        }
    }

    pub fn reset(&mut self) {
        self.goal_yaw = DEFAULT_YAW;
        self.goal_pitch = DEFAULT_PITCH;
        self.goal_distance = DEFAULT_DISTANCE;
        self.goal_target = Vec3::ZERO;
    }

    pub fn zoom(&self) -> f32 {
        REFERENCE_DISTANCE / self.distance
    }

    pub fn position(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        self.target + vec3(cp * cy, sp, cp * sy) * self.distance
    }

    pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let forward = (self.target - self.position()).normalize();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let right = if right == Vec3::ZERO { Vec3::X } else { right };
        let up = right.cross(forward);
        (forward, right, up)
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.goal_yaw += dx * 0.006;
        self.goal_pitch = (self.goal_pitch + dy * 0.006).clamp(-1.45, 1.45);
    }

    pub fn pan_screen(&mut self, dx: f32, dy: f32) {
        let (_, right, up) = self.basis();
        let k = self.distance * (self.fovy * 0.5).tan() * 2.0 / screen_height().max(1.0);
        self.goal_target += (-right * dx + up * dy) * k;
    }

    pub fn pan_ground(&mut self, forward_amount: f32, right_amount: f32) {
        let (sy, cy) = self.goal_yaw.sin_cos();
        let forward = -vec3(cy, 0.0, sy);
        let right = vec3(-sy, 0.0, cy);
        self.goal_target += (forward * forward_amount + right * right_amount) * self.goal_distance;
    }

    pub fn zoom_by(&mut self, steps: f32, focus: Option<Vec3>) {
        let old = self.goal_distance;
        let new = (old * (1.0 - steps * 0.12)).clamp(MIN_DISTANCE, MAX_DISTANCE);
        if let Some(f) = focus {
            let t = 1.0 - new / old;
            self.goal_target += (f - self.goal_target) * t;
        }
        self.goal_distance = new;
    }

    pub fn set_target(&mut self, target: Vec3) {
        self.goal_target = target;
    }

    pub fn update(&mut self, dt: f32) {
        if self.auto_rotate {
            self.goal_yaw += dt * 0.06;
        }
        let k = 1.0 - (-dt * 9.0).exp();
        let k_zoom = 1.0 - (-dt * 4.0).exp();
        self.yaw += (self.goal_yaw - self.yaw) * k;
        self.pitch += (self.goal_pitch - self.pitch) * k;
        self.target += (self.goal_target - self.target) * k;
        let ld = self.distance.ln() + (self.goal_distance.ln() - self.distance.ln()) * k_zoom;
        self.distance = ld.exp();
    }

    pub fn camera3d(&self) -> Camera3D {
        Camera3D {
            position: self.position(),
            target: self.target,
            up: Vec3::Y,
            fovy: self.fovy,
            z_near: (self.distance * 0.002).max(0.05),
            z_far: 200_000.0,
            ..Default::default()
        }
    }

    pub fn screen_ray(&self, screen: Vec2) -> (Vec3, Vec3) {
        let (forward, right, up) = self.basis();
        let h = (self.fovy * 0.5).tan();
        let aspect = screen_width() / screen_height().max(1.0);
        let nx = (screen.x / screen_width()) * 2.0 - 1.0;
        let ny = 1.0 - (screen.y / screen_height()) * 2.0;
        let dir = (forward + right * (nx * h * aspect) + up * (ny * h)).normalize();
        (self.position(), dir)
    }

    pub fn pixel_angle(&self) -> f32 {
        (self.fovy * 0.5).tan() * 2.0 / screen_height().max(1.0)
    }

    pub fn ground_point(&self, screen: Vec2) -> Option<Vec3> {
        let (o, d) = self.screen_ray(screen);
        if d.y.abs() < 1e-4 {
            return None;
        }
        let t = (self.target.y - o.y) / d.y;
        if t <= 0.0 {
            return None;
        }
        let p = o + d * t;
        if (p - self.target).length() > self.distance * 3.0 {
            None
        } else {
            Some(p)
        }
    }
}
