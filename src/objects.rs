use macroquad::rand::gen_range;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectType {
    Star,
    Planet,
    GalaxyCenter,
    DarkMatter,
    #[allow(dead_code)]
    DarkEnergy,
    BlackHole,
    NeutronStar,
    Pulsar,
    WhiteDwarf,
    Comet,
    Asteroid,
    Nebula,
}

impl ObjectType {
    pub fn name(self) -> &'static str {
        match self {
            ObjectType::Star => "Star",
            ObjectType::Planet => "Planet",
            ObjectType::GalaxyCenter => "Galaxy Center",
            ObjectType::DarkMatter => "Dark Matter",
            ObjectType::DarkEnergy => "Dark Energy",
            ObjectType::BlackHole => "Black Hole",
            ObjectType::NeutronStar => "Neutron Star",
            ObjectType::Pulsar => "Pulsar",
            ObjectType::WhiteDwarf => "White Dwarf",
            ObjectType::Comet => "Comet",
            ObjectType::Asteroid => "Asteroid",
            ObjectType::Nebula => "Nebula",
        }
    }
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy)]
pub struct NebulaParticle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct CelestialObject {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
    pub radius: f32,
    pub mass: f32,
    pub object_type: ObjectType,
    pub color: (u8, u8, u8),
    pub creation_time: f64,
    pub luminosity: f32,
    pub temperature: f32,
    #[allow(dead_code)]
    pub is_black_hole: bool,
    pub event_horizon_radius: f32,
    pub rotation_period: f32,
    pub magnetic_field: f32,
    pub nebula_density: f32,
    pub nebula_particles: Vec<NebulaParticle>,
    pub visual_seed: f32,
}

impl CelestialObject {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: f32, y: f32, z: f32,
        vx: f32, vy: f32, vz: f32,
        radius: f32, mass: f32,
        object_type: ObjectType, creation_time: f64,
    ) -> Self {
        let (color, luminosity, temperature, is_black_hole, rotation_period, magnetic_field, nebula_density, nebula_particles) =
            match object_type {
                ObjectType::Star => {
                    let temp = gen_range(3500.0, 8000.0);
                    let color = temperature_to_color(temp);
                    (color, mass / 50000.0, temp, false, 0.0, 0.0, 0.0, Vec::new())
                }
                ObjectType::Planet => {
                    let colors = [
                        (80, 120, 200),
                        (160, 100, 60),
                        (100, 160, 80),
                        (180, 160, 120),
                    ];
                    let color = colors[gen_range(0, colors.len())];
                    (color, 0.0, gen_range(150.0, 250.0), false, 0.0, 0.0, 0.0, Vec::new())
                }
                ObjectType::GalaxyCenter => ((240, 240, 160), mass / 5000.0, 6000.0, false, 0.0, 0.0, 0.0, Vec::new()),
                ObjectType::DarkMatter => ((60, 60, 140), 0.0, 0.0, false, 0.0, 0.0, 0.0, Vec::new()),
                ObjectType::DarkEnergy => ((120, 40, 180), 0.0, 0.0, false, 0.0, 0.0, 0.0, Vec::new()),
                ObjectType::BlackHole => ((10, 10, 10), 0.0, 0.0, true, 0.0, 0.0, 0.0, Vec::new()),
                ObjectType::NeutronStar => (
                    (220, 220, 255), mass / 100000.0, 1000000.0, false,
                    gen_range(0.001, 0.01), gen_range(1e8, 1e12), 0.0, Vec::new(),
                ),
                ObjectType::Pulsar => (
                    (200, 240, 255), mass / 80000.0, 800000.0, false,
                    gen_range(0.001, 0.1), gen_range(1e10, 1e13), 0.0, Vec::new(),
                ),
                ObjectType::WhiteDwarf => ((240, 240, 255), mass / 100000.0, 10000.0, false, 0.0, 0.0, 0.0, Vec::new()),
                ObjectType::Comet => ((180, 200, 240), 0.0, 180.0, false, 0.0, 0.0, 0.0, Vec::new()),
                ObjectType::Asteroid => ((120, 120, 120), 0.0, 0.0, false, 0.0, 0.0, 0.0, Vec::new()),
                ObjectType::Nebula => {
                    let nebula_colors = [
                        (200, 100, 150),
                        (100, 150, 200),
                        (150, 100, 200),
                        (200, 150, 100),
                        (100, 200, 150),
                    ];
                    let color = nebula_colors[gen_range(0, nebula_colors.len())];
                    let density = gen_range(0.5, 1.5);

                    let particle_count = gen_range(30, 80);
                    let mut particles = Vec::with_capacity(particle_count);
                    for _ in 0..particle_count {
                        particles.push(NebulaParticle {
                            x: gen_range(-radius, radius) * 2.0,
                            y: gen_range(-radius, radius) * 2.0,
                            z: gen_range(-radius, radius) * 0.8,
                            size: gen_range(0.5, 2.0) * density,
                        });
                    }
                    (color, 0.0, gen_range(100.0, 300.0), false, 0.0, 0.0, density, particles)
                }
            };

        let event_horizon_radius = if is_black_hole {
            2.0 * 6.67430e-11_f32 * mass / (299792458.0 * 299792458.0)
        } else {
            0.0
        };

        CelestialObject {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            x, y, z,
            vx, vy, vz,
            ax: 0.0, ay: 0.0, az: 0.0,
            radius,
            mass,
            object_type,
            color,
            creation_time,
            luminosity,
            temperature,
            is_black_hole,
            event_horizon_radius,
            rotation_period,
            magnetic_field,
            nebula_density,
            nebula_particles,
            visual_seed: gen_range(0.0, 1000.0),
        }
    }

    pub fn update(&mut self, time_scale: f32) {
        self.x += self.vx * time_scale * 0.1;
        self.y += self.vy * time_scale * 0.1;
        self.z += self.vz * time_scale * 0.1;

        if self.object_type == ObjectType::Nebula {
            let limit = self.radius * 2.0;
            for p in &mut self.nebula_particles {
                p.x += time_scale * 0.01;
                p.y += time_scale * 0.005;
                if p.x > limit {
                    p.x = -limit;
                }
                if p.y > limit {
                    p.y = -limit;
                }
            }
        }
    }

    #[inline]
    pub fn speed(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }

    pub fn get_age(&self, universe_age: f64) -> f64 {
        universe_age - self.creation_time
    }
}

fn temperature_to_color(temperature: f32) -> (u8, u8, u8) {
    let normalized_temp = temperature / 10000.0;

    if normalized_temp < 0.4 {
        (255, (100.0 + normalized_temp * 300.0) as u8, 50)
    } else if normalized_temp < 0.7 {
        (255, 220, 100)
    } else {
        (200, 220, 255)
    }
}
