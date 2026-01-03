use crate::objects::CelestialObject;
use crate::gravity::GravityEngine;

pub struct PhysicsEngine {
    pub gravity_engine: GravityEngine,
    pub max_speed: f32,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        PhysicsEngine {
            gravity_engine: GravityEngine::new(),
            max_speed: 100.0, //limit
        }
    }
    
    pub fn update_physics(&mut self, objects: &mut [CelestialObject], time_scale: f32) {
        // calculate gravity
        self.gravity_engine.calculate_gravitational_forces(objects);
        
        // 
        for obj in objects.iter_mut() {
            // convert (F = ma => a = F/m)
            if obj.mass > 0.0 {
                let ax = obj.ax / obj.mass;
                let ay = obj.ay / obj.mass;
                
                obj.vx += ax * time_scale * self.gravity_engine.time_step_factor;
                obj.vy += ay * time_scale * self.gravity_engine.time_step_factor;
                
               // let speed: f32 = (obj.vx * obj.vx + obj.vy * obj.vy);

               //
                let speed: f32 = (obj.vx * obj.vx + obj.vy * obj.vy).sqrt();
                if speed > self.max_speed {
                    obj.vx = obj.vx / speed * self.max_speed;
                    obj.vy = obj.vy / speed * self.max_speed;
                }
            }
        }
    }
}