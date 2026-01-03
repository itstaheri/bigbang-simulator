use std::time::Instant;
use rand::Rng;
use crate::objects::{CelestialObject, ObjectType};
use crate::physics::PhysicsEngine;
use crate::expansion::ExpansionEngine;
use crate::gravity::G;

pub struct Universe {
    pub width: f32,
    pub height: f32,
    pub objects: Vec<CelestialObject>,
    pub physics_engine: PhysicsEngine,
    pub expansion_engine: ExpansionEngine,
    pub start_time: Instant,
    pub age: f64,
    pub time_scale: f64,
}

impl Universe {
    pub fn new(width: f32, height: f32) -> Self {
        let mut universe = Universe {
            width,
            height,
            objects: Vec::new(),
            physics_engine: PhysicsEngine::new(),
            expansion_engine: ExpansionEngine::new(),
            start_time: Instant::now(),
            age: 0.0,
            time_scale: 0.1,
        };
        
        universe.create_stable_universe();
        universe.add_random_kick(); 
        universe
    }
    
    fn create_stable_universe(&mut self) {
        let mut rng = rand::thread_rng();
        
        self.create_central_galaxy();
        
        self.create_stable_solar_systems();
        
        self.create_small_objects();
    }
    
    fn create_central_galaxy(&mut self) {
        let mut rng = rand::thread_rng();
        
        let central_mass = 5e7;
        self.objects.push(CelestialObject::new(
            0.0, 0.0, 0.0, 0.0, 15.0, central_mass, ObjectType::GalaxyCenter, self.age
        ));
        // self.objects.push(CelestialObject::new(
        //     0.0, 0.0, 0.0, 0.0, 3.0, central_mass * 1, ObjectType::BlackHole, self.age
        // ));
        
        // Central black hole
        self.objects.push(CelestialObject::new(
            0.0, 0.0, 0.0, 0.0, 3.0, central_mass * 10.0, ObjectType::BlackHole, self.age
        ));
        
        // Stars in stable orbits around the center
        for i in 0..30 {
            let distance = 100.0 + i as f32 * 25.0;
            let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
            
            // Orbital speed for circular orbit (Keplers law)
            let orbital_speed = (G * central_mass / distance).sqrt() * 0.7;
            
            self.objects.push(CelestialObject::new(
                distance * angle.cos(),
                distance * angle.sin(),
                -orbital_speed * angle.sin(),// Tangent to orbit
                // -orbital_speed * angle.cos(),  
                orbital_speed * angle.cos(),
                rng.gen_range(1.0..2.5),
                rng.gen_range(5e4..2e5),
                ObjectType::Star,
                self.age,
            ));
        }
    }
    
    fn create_stable_solar_systems(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Create a few solar systems
        for system_num in 0..4 {
            let system_x = rng.gen_range(-300.0..300.0);
            let system_y = rng.gen_range(-300.0..300.0);
            
            // Central star
            let star_mass = rng.gen_range(1e6..5e6);
            let star_type = if rng.gen_bool(0.1) {
                ObjectType::WhiteDwarf
            } else {
                ObjectType::Star
            };
            
            self.objects.push(CelestialObject::new(
                system_x, system_y, 0.0, 0.0, 
                match star_type {
                    ObjectType::WhiteDwarf => rng.gen_range(1.0..1.5),
                    _ => rng.gen_range(2.0..4.0),
                },
                star_mass,
                star_type,
                self.age,
            ));
            
            // Planets with stable orbits
            for planet_idx in 0..rng.gen_range(2..4) {
                let orbit_radius = 25.0 + planet_idx as f32 * 15.0;
                let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
                
                // Circular orbit speed
                let orbital_speed = (G * star_mass / orbit_radius).sqrt();
                
                self.objects.push(CelestialObject::new(
                    system_x + orbit_radius * angle.cos(),
                    system_y + orbit_radius * angle.sin(),
                    -orbital_speed * angle.sin(),
                    orbital_speed * angle.cos(),
                    rng.gen_range(0.5..1.2),
                    rng.gen_range(1e3..5e3),
                    ObjectType::Planet,
                    self.age,
                ));
            }
            
            // Add a neutron star or pulsar to some systems
            if rng.gen_bool(0.2) {
                let exotic_type = if rng.gen_bool(0.5) {
                    ObjectType::NeutronStar
                } else {
                    ObjectType::Pulsar
                };
                
                let exotic_distance = 50.0;
                let exotic_angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
                let exotic_speed = (G * star_mass / exotic_distance).sqrt() * 0.8;
                
                self.objects.push(CelestialObject::new(
                    system_x + exotic_distance * exotic_angle.cos(),
                    system_y + exotic_distance * exotic_angle.sin(),
                    -exotic_speed * exotic_angle.sin(),
                    exotic_speed * exotic_angle.cos(),
                    rng.gen_range(0.3..0.6),
                    rng.gen_range(5e4..1e5),
                    exotic_type,
                    self.age,
                ));
            }
        }
    }
    
    fn create_small_objects(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Asteroids
        for _ in 0..20 {
            let x = rng.gen_range(-400.0..400.0);
            let y = rng.gen_range(-400.0..400.0);
            
            // Random initial velocity
            let speed = rng.gen_range(0.05..0.2);
            let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
            //----------------------------
            self.objects.push(CelestialObject::new(
                x, y,
                speed * angle.cos(),
                speed * angle.sin(),
                rng.gen_range(0.1..0.3),
                rng.gen_range(0.5..5.0),
                ObjectType::Asteroid,
                self.age,
            ));
                        //----------------------------

        }
        
        // Comets
        for _ in 0..8 {
            let x = rng.gen_range(-500.0..500.0);
            let y = rng.gen_range(-500.0..500.0);
            
            let speed = rng.gen_range(0.1..0.3);
            let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
            
            self.objects.push(CelestialObject::new(
                x, y,
                speed * angle.cos(),
                speed * angle.sin(),
                rng.gen_range(0.2..0.5),
                rng.gen_range(1.0..3.0),
                ObjectType::Comet,
                self.age,
            ));
        }
        
        // Dark matter
        for _ in 0..15 {
            let x = rng.gen_range(-600.0..600.0);
            let y = rng.gen_range(-600.0..600.0);
            
            self.objects.push(CelestialObject::new(
                x, y, 0.0, 0.0,
                rng.gen_range(1.0..2.0),
                rng.gen_range(1e4..5e4),
                ObjectType::DarkMatter,
                self.age,
            ));
        }
    }
    
    fn add_random_kick(&mut self) {
        let mut rng = rand::thread_rng();
        
        for obj in self.objects.iter_mut() {
            // Only give small/medium objects a little kick
            if obj.mass < 1e5 && !matches!(obj.object_type, ObjectType::GalaxyCenter) {
                let kick_strength = 0.05;
                obj.vx += rng.gen_range(-kick_strength..kick_strength);
                obj.vy += rng.gen_range(-kick_strength..kick_strength);
            }
        }
    }
    
    pub fn update(&mut self, expansion_speed: f32) {
        self.age += 0.001 * self.time_scale;
        
        // 1. Apply gravity
        self.physics_engine.update_physics(&mut self.objects, self.time_scale as f32);
        
        self.expansion_engine.apply_expansion(&mut self.objects, expansion_speed * 0.01, self.age as f32);
        
        // 3. Update positions
        for obj in &mut self.objects {
            obj.update(self.time_scale as f32);
        }
        
        // 4. Remove objects that wandered too far (optional)
        self.remove_distant_objects();
    }
    
    fn remove_distant_objects(&mut self) {
        let max_distance = 2000.0;
        self.objects.retain(|obj| {
            let distance = (obj.x * obj.x + obj.y * obj.y).sqrt();
            distance < max_distance
        });
    }
    
    pub fn get_object_at_position(&self, x: f32, y: f32, zoom: f32) -> Option<(usize, &CelestialObject)> {
        let search_radius = 25.0 / zoom.max(0.1);
        
        for (i, obj) in self.objects.iter().enumerate() {
            let dx = obj.x - x;
            let dy = obj.y - y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < obj.radius + search_radius {
                return Some((i, obj));
            }
        }
        None
    }
    
    pub fn get_object_by_index(&self, index: usize) -> Option<&CelestialObject> {
        self.objects.get(index)
    }
}