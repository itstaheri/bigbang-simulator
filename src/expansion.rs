use crate::objects::CelestialObject;

pub struct ExpansionEngine {
    pub expansion_rate: f32,
    pub dark_energy_density: f32,
}

impl ExpansionEngine {
    pub fn new() -> Self {
        ExpansionEngine {
            expansion_rate: 0.0000001,  // to keep orbits stable
            dark_energy_density: 0.001,  // just a hint of dark energy
        }
    }
    
    pub fn apply_expansion(&self, objects: &mut [CelestialObject], speed_factor: f32, age: f32) {
        let current_expansion_rate = self.expansion_rate * speed_factor * 0.0001;
        
        if current_expansion_rate < 1e-10 {
            return;
        }
        
        // Only affect objects really far from center
        for obj in objects {
            let distance_from_center = (obj.x * obj.x + obj.y * obj.y).sqrt();
            
            if distance_from_center > 1000.0 {
                let expansion_factor = 1.0 + current_expansion_rate;
                obj.x *= expansion_factor;
                obj.y *= expansion_factor;
            }
        }
    }
}