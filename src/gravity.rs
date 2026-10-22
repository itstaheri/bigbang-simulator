use crate::objects::CelestialObject;

pub const G: f32 = 6.67430e-5_f32;

pub struct GravityEngine {
    pub softening_length: f32,

    pub time_step_factor: f32,

    pub enabled: bool,
}

impl GravityEngine {
    pub fn new() -> Self {
        GravityEngine {
            softening_length: 15.0,
            time_step_factor: 0.02,
            enabled: true,
        }
    }

    pub fn calculate_gravitational_forces(&self, objects: &mut [CelestialObject]) {
        if !self.enabled {
            return;
        }

        for obj in objects.iter_mut() {
            obj.ax = 0.0;
            obj.ay = 0.0;
            obj.az = 0.0;
        }

        let soft_sq = self.softening_length * self.softening_length;
        let n = objects.len();

        for i in 0..n {
            let (head, tail) = objects.split_at_mut(i + 1);
            let a = &mut head[i];
            let (ax0, ay0, az0) = (a.x, a.y, a.z);
            let (mass_a, radius_a) = (a.mass, a.radius);
            let (mut acc_x, mut acc_y, mut acc_z) = (0.0f32, 0.0f32, 0.0f32);

            for b in tail.iter_mut() {
                let dx = b.x - ax0;
                let dy = b.y - ay0;
                let dz = b.z - az0;

                let distance_sq = dx * dx + dy * dy + dz * dz + soft_sq;
                let distance = distance_sq.sqrt();

                if distance < (radius_a + b.radius) * 2.0 {
                    continue;
                }

                let inv_r3 = G / (distance_sq * distance);
                let k_a = inv_r3 * b.mass;
                let k_b = inv_r3 * mass_a;

                acc_x += k_a * dx;
                acc_y += k_a * dy;
                acc_z += k_a * dz;

                b.ax -= k_b * dx;
                b.ay -= k_b * dy;
                b.az -= k_b * dz;
            }

            a.ax += acc_x;
            a.ay += acc_y;
            a.az += acc_z;
        }
    }

    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[allow(dead_code)]
    pub fn set_time_step_factor(&mut self, factor: f32) {
        self.time_step_factor = factor.clamp(0.001, 0.1);
    }
}
