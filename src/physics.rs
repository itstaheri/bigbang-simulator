use crate::gravity::GravityEngine;
use crate::objects::CelestialObject;

pub struct PhysicsEngine {
    pub gravity_engine: GravityEngine,

    pub max_speed: f32,

    pub enable_collisions: bool,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        PhysicsEngine {
            gravity_engine: GravityEngine::new(),
            max_speed: 100.0,
            enable_collisions: true,
        }
    }

    pub fn update_physics(&mut self, objects: &mut [CelestialObject], time_scale: f32) {
        self.gravity_engine.calculate_gravitational_forces(objects);

        let step = time_scale * self.gravity_engine.time_step_factor;
        let max_speed_sq = self.max_speed * self.max_speed;

        for obj in objects.iter_mut() {
            if obj.mass > 0.0 {
                obj.vx += obj.ax * step;
                obj.vy += obj.ay * step;
                obj.vz += obj.az * step;

                let speed_sq = obj.vx * obj.vx + obj.vy * obj.vy + obj.vz * obj.vz;
                if speed_sq > max_speed_sq {
                    let k = self.max_speed / speed_sq.sqrt();
                    obj.vx *= k;
                    obj.vy *= k;
                    obj.vz *= k;
                }
            }
        }

        if self.enable_collisions {
            self.handle_collisions(objects);
        }
    }

    fn handle_collisions(&self, objects: &mut [CelestialObject]) {
        let n = objects.len();
        let restitution = 0.8;
        let correction_percent = 0.2;

        for i in 0..n {
            let (head, tail) = objects.split_at_mut(i + 1);
            let a = &mut head[i];

            for b in tail.iter_mut() {
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let dz = b.z - a.z;
                let min_distance = a.radius + b.radius;
                let threshold = min_distance * 1.5;
                let distance_sq = dx * dx + dy * dy + dz * dz;

                if distance_sq >= threshold * threshold {
                    continue;
                }

                if distance_sq < 1e-12 {
                    continue;
                }

                let distance = distance_sq.sqrt();
                let nx = dx / distance;
                let ny = dy / distance;
                let nz = dz / distance;

                let velocity_along_normal =
                    (b.vx - a.vx) * nx + (b.vy - a.vy) * ny + (b.vz - a.vz) * nz;

                if velocity_along_normal > 0.0 {
                    continue;
                }

                let inv_mass_sum = 1.0 / a.mass + 1.0 / b.mass;

                let impulse = -(1.0 + restitution) * velocity_along_normal / inv_mass_sum;
                let (ix, iy, iz) = (impulse * nx, impulse * ny, impulse * nz);

                a.vx -= ix / a.mass;
                a.vy -= iy / a.mass;
                a.vz -= iz / a.mass;
                b.vx += ix / b.mass;
                b.vy += iy / b.mass;
                b.vz += iz / b.mass;

                let penetration = (min_distance - distance).max(0.0);
                if penetration > 0.0 {
                    let correction = penetration / inv_mass_sum * correction_percent;
                    let (cx, cy, cz) = (correction * nx, correction * ny, correction * nz);

                    a.x -= cx / a.mass;
                    a.y -= cy / a.mass;
                    a.z -= cz / a.mass;
                    b.x += cx / b.mass;
                    b.y += cy / b.mass;
                    b.z += cz / b.mass;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_max_speed(&mut self, max_speed: f32) {
        self.max_speed = max_speed.max(10.0);
    }

    #[allow(dead_code)]
    pub fn toggle_collisions(&mut self) {
        self.enable_collisions = !self.enable_collisions;
    }
}
