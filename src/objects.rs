use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectType {
    Star,
    Planet,
    GalaxyCenter,
    DarkMatter,
    DarkEnergy,
    BlackHole,
    NeutronStar,
    Pulsar,
    WhiteDwarf,
    Comet,
    Asteroid,
    // Galaxy
}

#[derive(Debug, Clone)]
pub struct CelestialObject {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ax: f32,
    pub ay: f32,
    pub radius: f32,
    pub mass: f32,
    pub object_type: ObjectType,
    pub color: (u8, u8, u8),
    pub creation_time: f64,
    pub luminosity: f32,
    pub temperature: f32,
    pub is_black_hole: bool,
    pub event_horizon_radius: f32,
    pub rotation_period: f32,
    pub magnetic_field: f32,
}

impl CelestialObject {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32, radius: f32, mass: f32, 
               object_type: ObjectType, creation_time: f64) -> Self {
        let mut rng = rand::thread_rng();
        
        let (color, luminosity, temperature, is_black_hole, rotation_period, magnetic_field) = 
            match object_type {
            ObjectType::Star => {
                let temp = rng.gen_range(3500.0..8000.0);
                let color = temperature_to_color(temp);
                (color, mass / 50000.0, temp, false, 0.0, 0.0)
            }
            ObjectType::Planet => {
                let colors = [
                    (80, 120, 200),   // blue
                    (160, 100, 60),   // brown
                    (100, 160, 80),   // green
                    (180, 160, 120),  // beige
                ];
                // let rand_index = rng.gen_range(0..colors.len());
                // let color = colors[rand_index];
                let color = colors[rng.gen_range(0..colors.len())];
                (color, 0.0, rng.gen_range(150.0..250.0), false, 0.0, 0.0)
            }
            ObjectType::GalaxyCenter => {
                ((240, 240, 160), mass / 5000.0, 6000.0, false, 0.0, 0.0)
            }
            ObjectType::DarkMatter => {
                ((60, 60, 140), 0.0, 0.0, false, 0.0, 0.0)  // dark purple
            }
            ObjectType::DarkEnergy => {
                ((120, 40, 180), 0.0, 0.0, false, 0.0, 0.0)  // purple
            }
            ObjectType::BlackHole => {
                ((10, 10, 10), 0.0, 0.0, true, 0.0, 0.0)  // black :) thats the fucking back hole!
            }
            ObjectType::NeutronStar => {
                ((220, 220, 255), mass / 100000.0, 1000000.0, false, 
                 rng.gen_range(0.001..0.01),  // very fast rotation
                 rng.gen_range(1e8..1e12))  
            }
            ObjectType::Pulsar => {
                ((200, 240, 255), mass / 80000.0, 800000.0, false,
                 rng.gen_range(0.001..0.1),   // Even  rotation
                 rng.gen_range(1e10..1e13))   // Even more ridiculous field
            }
            ObjectType::WhiteDwarf => {
                ((240, 240, 255), mass / 100000.0, 10000.0, false, 0.0, 0.0)
            }
            ObjectType::Comet => {
                ((180, 200, 240), 0.0, 180.0, false, 0.0, 0.0)
            }
            ObjectType::Asteroid => {
                ((120, 120, 120), 0.0, 0.0, false, 0.0, 0.0)  // gray rock
            }
        };
        
        // calculate event horizon for black holes  - if it is one -
        let event_horizon_radius = if is_black_hole {
            2.0 * 6.67430e-11_f32 * mass / (299792458.0 * 299792458.0) 
        } else {
            0.0
        };
        
        CelestialObject {
            x,
            y,
            vx,
            vy,
            ax: 0.0,
            ay: 0.0,
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
        }
    }
    
    pub fn update(&mut self, time_scale: f32) {
        // Update position 
        self.x += self.vx * time_scale * 0.1;
        self.y += self.vy * time_scale * 0.1;
    }
    
    pub fn get_age(&self, universe_age: f64) -> f64 {
        universe_age - self.creation_time
    }
}

// Make stars colorful based on temperature
fn temperature_to_color(temperature: f32) -> (u8, u8, u8) {
    let normalized_temp = temperature / 10000.0;
    
    if normalized_temp < 0.4 {
        // Red stars (coolest) 
        let r = 255;
        let g = (100.0 + normalized_temp * 300.0) as u8;
        let b = 50;
        (r, g, b)
    } else if normalized_temp < 0.7 {
        // Yellow stars (medium)
        let r = 255;
        let g = 220;
        let b = 100;
        (r, g, b)
    } else {
        // Blue-white stars (hottest)
        let r = 200;
        let g = 220;
        let b = 255;
        (r, g, b)
    }
}