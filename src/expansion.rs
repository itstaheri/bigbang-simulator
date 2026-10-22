use crate::objects::CelestialObject;

pub struct ExpansionEngine {
    pub expansion_rate: f32,

    pub dark_energy_density: f32,

    pub enabled: bool,

    pub universe_age: f32,
}

impl ExpansionEngine {
    pub fn new() -> Self {
        ExpansionEngine {
            expansion_rate: 0.0000001,
            dark_energy_density: 0.001,
            enabled: true,
            universe_age: 0.0,
        }
    }

    pub fn apply_expansion(&mut self, objects: &mut [CelestialObject], speed_factor: f32, age: f32) {
        if !self.enabled {
            return;
        }

        self.universe_age = age;

        let current_expansion_rate = self.expansion_rate * speed_factor * 0.0001;

        if current_expansion_rate < 1e-10 {
            return;
        }

        let dark_energy_factor = if self.dark_energy_density > 0.0 {
            1.0 + self.dark_energy_density * (self.universe_age / 1000.0).powi(2)
        } else {
            1.0
        };

        for obj in objects {
            let distance_sq = obj.x * obj.x + obj.y * obj.y + obj.z * obj.z;

            if distance_sq > 1000.0 * 1000.0 {
                let distance_from_center = distance_sq.sqrt();
                let factor = (1.0 + current_expansion_rate * (distance_from_center / 1000.0))
                    * dark_energy_factor;
                obj.x *= factor;
                obj.y *= factor;
                obj.z *= factor;
            }
        }
    }

    #[allow(dead_code)]
    pub fn hubble_parameter(&self) -> f32 {
        self.expansion_rate * (1.0 + 0.5 * (self.universe_age / 1000.0).powi(2))
    }

    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[allow(dead_code)]
    pub fn set_dark_energy_density(&mut self, density: f32) {
        self.dark_energy_density = density.clamp(0.0, 0.01);
    }
}
