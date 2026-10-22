use std::f32::consts::PI;

use macroquad::math::{vec3, Vec3};
use macroquad::rand::gen_range;

use crate::expansion::ExpansionEngine;
use crate::gravity::G;
use crate::objects::{CelestialObject, ObjectType};
use crate::physics::PhysicsEngine;

pub struct Universe {
    #[allow(dead_code)]
    pub width: f32,
    #[allow(dead_code)]
    pub height: f32,
    pub objects: Vec<CelestialObject>,
    pub physics_engine: PhysicsEngine,
    pub expansion_engine: ExpansionEngine,
    pub age: f64,
    pub time_scale: f64,
}

#[inline]
fn chance(p: f32) -> bool {
    gen_range(0.0f32, 1.0) < p
}

fn plane_basis(n: Vec3) -> (Vec3, Vec3) {
    let helper = if n.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
    let u = n.cross(helper).normalize();
    let w = n.cross(u);
    (u, w)
}

fn random_tilted_normal(max_tilt: f32) -> Vec3 {
    let tilt = gen_range(0.0, max_tilt);
    let azimuth = gen_range(0.0, 2.0 * PI);
    vec3(tilt.sin() * azimuth.cos(), tilt.sin() * azimuth.sin(), tilt.cos())
}

impl Universe {
    pub fn new(width: f32, height: f32) -> Self {
        let mut universe = Universe {
            width,
            height,
            objects: Vec::new(),
            physics_engine: PhysicsEngine::new(),
            expansion_engine: ExpansionEngine::new(),
            age: 0.0,
            time_scale: 0.1,
        };

        universe.create_stable_universe();
        universe.add_random_kick();
        universe.remove_distant_objects();
        universe
    }

    fn create_stable_universe(&mut self) {
        self.create_central_galaxy();
        self.create_stable_solar_systems();
        self.create_nebulae();
        self.create_small_objects();
    }

    fn push(&mut self, pos: Vec3, vel: Vec3, radius: f32, mass: f32, t: ObjectType) -> &mut CelestialObject {
        self.objects.push(CelestialObject::new(
            pos.x, pos.y, pos.z, vel.x, vel.y, vel.z, radius, mass, t, self.age,
        ));
        self.objects.last_mut().unwrap()
    }

    fn create_nebulae(&mut self) {
        for _ in 0..8 {
            let x = gen_range(-400.0, 400.0);
            let y = gen_range(-400.0, 400.0);

            let distance_from_center = (x * x + y * y as f32).sqrt();
            if distance_from_center < 150.0 {
                continue;
            }

            let z = gen_range(-80.0, 80.0);
            let radius = gen_range(15.0, 40.0);
            let mass = radius * 1000.0;
            let vel = vec3(gen_range(-0.01, 0.01), gen_range(-0.01, 0.01), gen_range(-0.004, 0.004));

            self.push(vec3(x, y, z), vel, radius, mass, ObjectType::Nebula);
        }

        let special_colors = [
            (200, 100, 150),
            (100, 150, 200),
            (150, 100, 200),
        ];

        for (i, color) in special_colors.iter().enumerate() {
            let angle = i as f32 * 2.0 * PI / 3.0;
            let distance = 500.0;
            let pos = vec3(distance * angle.cos(), distance * angle.sin(), gen_range(-40.0, 40.0));

            let radius = gen_range(30.0, 50.0);
            let mass = radius * 1500.0;

            let nebula = self.push(pos, Vec3::ZERO, radius, mass, ObjectType::Nebula);
            nebula.color = *color;
        }
    }

    fn create_central_galaxy(&mut self) {
        let central_mass = 5e7;
        self.push(Vec3::ZERO, Vec3::ZERO, 15.0, central_mass, ObjectType::GalaxyCenter);

        self.push(Vec3::ZERO, Vec3::ZERO, 3.0, central_mass * 10.0, ObjectType::BlackHole);

        for i in 0..300 {
            let distance = 100.0 + i as f32 * 25.0;
            let angle = gen_range(0.0, 2.0 * PI);

            let orbital_speed = (G * central_mass / distance).sqrt() * 0.7;

            let half_thickness = 4.0 + distance * 0.012;
            let z = gen_range(-1.0, 1.0) * gen_range(0.0, 1.0) * half_thickness;

            self.push(
                vec3(distance * angle.cos(), distance * angle.sin(), z),
                vec3(-orbital_speed * angle.sin(), orbital_speed * angle.cos(), 0.0),
                gen_range(1.0, 2.5),
                gen_range(5e4, 2e5),
                ObjectType::Star,
            );
        }
    }

    fn create_stable_solar_systems(&mut self) {
        for _ in 0..4 {
            let system = vec3(gen_range(-300.0, 300.0), gen_range(-300.0, 300.0), gen_range(-40.0, 40.0));

            let star_mass = gen_range(1e6, 5e6);
            let star_type = if chance(0.1) { ObjectType::WhiteDwarf } else { ObjectType::Star };
            let star_radius = match star_type {
                ObjectType::WhiteDwarf => gen_range(1.0, 1.5),
                _ => gen_range(2.0, 4.0),
            };
            self.push(system, Vec3::ZERO, star_radius, star_mass, star_type);

            let normal = random_tilted_normal(0.45);
            let (u, w) = plane_basis(normal);

            let planet_count = gen_range(2, 4);
            for planet_idx in 0..planet_count {
                let orbit_radius = 25.0 + planet_idx as f32 * 15.0;
                let angle = gen_range(0.0, 2.0 * PI);
                let orbital_speed = (G * star_mass / orbit_radius).sqrt();

                let radial = u * angle.cos() + w * angle.sin();
                let tangent = -u * angle.sin() + w * angle.cos();

                self.push(
                    system + radial * orbit_radius,
                    tangent * orbital_speed,
                    gen_range(0.5, 1.2),
                    gen_range(1e3, 5e3),
                    ObjectType::Planet,
                );
            }

            if chance(0.2) {
                let exotic_type = if chance(0.5) { ObjectType::NeutronStar } else { ObjectType::Pulsar };

                let exotic_distance = 50.0;
                let angle = gen_range(0.0, 2.0 * PI);
                let speed = (G * star_mass / exotic_distance).sqrt() * 0.8;
                let radial = u * angle.cos() + w * angle.sin();
                let tangent = -u * angle.sin() + w * angle.cos();

                self.push(
                    system + radial * exotic_distance,
                    tangent * speed,
                    gen_range(0.3, 0.6),
                    gen_range(5e4, 1e5),
                    exotic_type,
                );
            }
        }
    }

    fn flattened_direction(flatness: f32) -> Vec3 {
        let angle = gen_range(0.0, 2.0 * PI);
        let elevation = gen_range(-1.0, 1.0) * flatness;
        vec3(angle.cos() * elevation.cos(), angle.sin() * elevation.cos(), elevation.sin())
    }

    fn create_small_objects(&mut self) {
        for _ in 0..20 {
            let pos = vec3(gen_range(-400.0, 400.0), gen_range(-400.0, 400.0), gen_range(-60.0, 60.0));
            let speed = gen_range(0.05, 0.2);
            self.push(
                pos,
                Self::flattened_direction(0.3) * speed,
                gen_range(0.1, 0.3),
                gen_range(0.5, 5.0),
                ObjectType::Asteroid,
            );
        }

        for _ in 0..8 {
            let pos = vec3(gen_range(-500.0, 500.0), gen_range(-500.0, 500.0), gen_range(-90.0, 90.0));
            let speed = gen_range(0.1, 0.3);
            self.push(
                pos,
                Self::flattened_direction(0.6) * speed,
                gen_range(0.2, 0.5),
                gen_range(1.0, 3.0),
                ObjectType::Comet,
            );
        }

        for _ in 0..15 {
            let pos = vec3(gen_range(-600.0, 600.0), gen_range(-600.0, 600.0), gen_range(-200.0, 200.0));
            self.push(pos, Vec3::ZERO, gen_range(1.0, 2.0), gen_range(1e4, 5e4), ObjectType::DarkMatter);
        }
    }

    fn add_random_kick(&mut self) {
        for obj in self.objects.iter_mut() {
            if obj.mass < 1e5 && !matches!(obj.object_type, ObjectType::GalaxyCenter | ObjectType::Nebula) {
                let kick = 0.05;
                obj.vx += gen_range(-kick, kick);
                obj.vy += gen_range(-kick, kick);
                obj.vz += gen_range(-kick, kick) * 0.3;
            }
        }
    }

    pub fn update(&mut self, expansion_speed: f32, time_multiplier: f32) {
        if time_multiplier <= 0.0 {
            return;
        }

        let ts = self.time_scale as f32 * time_multiplier;
        self.age += 0.001 * self.time_scale * time_multiplier as f64;

        self.physics_engine.update_physics(&mut self.objects, ts);

        self.expansion_engine
            .apply_expansion(&mut self.objects, expansion_speed * time_multiplier * 0.01, self.age as f32);

        for obj in &mut self.objects {
            obj.update(ts);
        }

        self.remove_distant_objects();
    }

    fn remove_distant_objects(&mut self) {
        let max_distance_sq = 2000.0f32 * 2000.0;
        self.objects.retain(|obj| {
            let d = obj.x * obj.x + obj.y * obj.y + obj.z * obj.z;
            d < max_distance_sq
        });
    }

    pub fn pick(&self, origin: Vec3, dir: Vec3, pixel_angle: f32) -> Option<u64> {
        let mut best: Option<(f32, f32, u64)> = None;

        for obj in &self.objects {
            let p = crate::rendering::world_pos(obj);
            let to = p - origin;
            let t = to.dot(dir);
            if t <= 0.0 {
                continue;
            }
            let perp_sq = (to - dir * t).length_squared();
            let visual = crate::rendering::visual_radius(obj);
            let pick_radius = visual.max(t * pixel_angle * 14.0);
            if perp_sq > pick_radius * pick_radius {
                continue;
            }
            let mut score = perp_sq.sqrt() / pick_radius;
            if matches!(obj.object_type, ObjectType::Nebula | ObjectType::DarkMatter | ObjectType::DarkEnergy) {
                score += 1.0;
            }
            let better = match best {
                None => true,
                Some((s, d, _)) => score < s - 0.05 || ((score - s).abs() <= 0.05 && t < d),
            };
            if better {
                best = Some((score, t, obj.id));
            }
        }
        best.map(|(_, _, id)| id)
    }

    pub fn get_object_by_id(&self, id: u64) -> Option<&CelestialObject> {
        self.objects.iter().find(|o| o.id == id)
    }
}
