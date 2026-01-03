use crate::objects::{CelestialObject, ObjectType};

// gravitational constant
pub const G: f32 = 6.67430e-5_f32;

pub struct GravityEngine {
    pub softening_length: f32,  
    pub time_step_factor: f32,  
}
//hint :
 // Safety cushion for gravity - stops objects from 
                                 // getting infinite attraction when they get too close
                                 // (Without this, two close stars would pull each other
                                 // with INFINITE force and break the simulation!)
                                 // so we Simulation speed controller - smaller steps = more stable,
                                 // bigger steps = faster but can get wobbly
                                 // (Think of it like "slow motion" for accurate physics)

impl GravityEngine {
    pub fn new() -> Self {
        GravityEngine {
            softening_length: 15.0,  // "Fudge factor" for numerical stability
            time_step_factor: 0.02,   // Smaller steps = more stability
        }
    }
    
    pub fn calculate_gravitational_forces(&self, objects: &mut [CelestialObject]) {
        let n = objects.len();
        
        // Zero out accelerations first
        for obj in objects.iter_mut() {
            obj.ax = 0.0;
            obj.ay = 0.0;
        }
        
        // Calculate forces between every pair of objects
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = objects[j].x - objects[i].x;
                let dy = objects[j].y - objects[i].y;
                
                // Distance with softening to avoid division by zero
                let distance_sq = dx * dx + dy * dy + self.softening_length.powi(2);
                let distance = distance_sq.sqrt();
                
                // Skip if objects are about to collide
                if distance < (objects[i].radius + objects[j].radius) * 2.0 {
                    continue;
                }
                
                // Newtons law of gravitation
                let force_magnitude = G / distance_sq;
                
                objects[i].ax += force_magnitude * objects[j].mass * dx / distance;
                objects[i].ay += force_magnitude * objects[j].mass * dy / distance;
                
                objects[j].ax -= force_magnitude * objects[i].mass * dx / distance;
                objects[j].ay -= force_magnitude * objects[i].mass * dy / distance;
            }
        }
    }
}