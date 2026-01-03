use macroquad::prelude::*;
use crate::universe::Universe;
use crate::objects::{ObjectType, CelestialObject};

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }
    
    // Helper 
    fn measure_text_width(&self, text: &str, font_size: f32) -> f32 {
        measure_text(text, None, font_size as u16, 1.0).width
    }
    
    fn draw_text_wrapped(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color, max_width: f32) -> f32 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();
        let mut current_y = y;
        let line_height = font_size * 1.2;
        
        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };
            
            let test_width = self.measure_text_width(&test_line, font_size);
            
            if test_width <= max_width {
                current_line = test_line;
            } else {
                // Draw the current line
                if !current_line.is_empty() {
                    draw_text(&current_line, x, current_y, font_size, color);
                    current_y += line_height;
                }
                current_line = word.to_string();
            }
        }
        
        if !current_line.is_empty() {
            draw_text(&current_line, x, current_y, font_size, color);
            current_y += line_height;
        }
        
        current_y - y  // Total text height
    }
    
    // Main drwing function - orchestrates the whole cosmic show
    pub fn draw_universe(&self, universe: &Universe, zoom: f32, offset_x: f32, offset_y: f32, 
                        screen_width: f32, screen_height: f32) {
        let center_x = screen_width / 2.0 + offset_x;
        let center_y = screen_height / 2.0 + offset_y;
        
        self.draw_subtle_background(center_x, center_y, zoom, screen_width, screen_height);
        
        // Draw orbits when user zoomed in
        if zoom > 1.5 {
            self.draw_stable_orbits(&universe.objects, center_x, center_y, zoom);
        }
        
        // Draw dark matter/energy first
        for obj in &universe.objects {
            if matches!(obj.object_type, 
                ObjectType::DarkEnergy | 
                ObjectType::DarkMatter) {
                self.draw_stable_object(obj, center_x, center_y, zoom);
            }
        }
        
        // Then draw the main attractions
        for obj in &universe.objects {
            if !matches!(obj.object_type, 
                ObjectType::DarkEnergy | 
                ObjectType::DarkMatter) {
                self.draw_stable_object(obj, center_x, center_y, zoom);
            }
        }
        
        // Draw connections between close objects when really zoomed in
        if zoom > 2.5 {
            self.draw_object_connections(&universe.objects, center_x, center_y, zoom);
        }
    }
    
    // Create a subtle starry background
    fn draw_subtle_background(&self, center_x: f32, center_y: f32, zoom: f32, screen_width: f32, screen_height: f32) {
        let star_count = 50;
        for i in 0..star_count {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / star_count as f32;
            let distance = 3000.0 / zoom;  // Background stars get closer when we zoom in
            let x = center_x + distance * angle.cos();
            let y = center_y + distance * angle.sin();
            
            // Make sure stars are actually on screen
            if x >= 0.0 && x <= screen_width && y >= 0.0 && y <= screen_height {
                let brightness = 0.05 + (i as f32 * 0.02).sin().abs() * 0.1;
                draw_circle(x, y, 0.5, Color::new(brightness, brightness, brightness, 0.5));
            }
        }
    }
    
    // Draw orbital paths 
    fn draw_stable_orbits(&self, objects: &[CelestialObject], center_x: f32, center_y: f32, zoom: f32) {
        let base_line_thickness = if zoom > 3.0 {
            1.2
        } else if zoom > 2.0 {
            0.8
        } else if zoom > 1.5 {
            0.6
        } else {
            0.4
        };
        
        let base_alpha = if zoom > 3.0 {
            0.25
        } else if zoom > 2.0 {
            0.2
        } else {
            0.15
        };
        
        // Find stars and draw orbits for their planets
        for star in objects.iter().filter(|obj| matches!(obj.object_type, ObjectType::Star | 
            ObjectType::WhiteDwarf | 
            ObjectType::NeutronStar | 
            ObjectType::Pulsar)) {
            let star_x = center_x + star.x * zoom;
            let star_y = center_y + star.y * zoom;
            
            for planet in objects.iter().filter(|obj| matches!(obj.object_type, ObjectType::Planet)) {
                let dx = planet.x - star.x;
                let dy = planet.y - star.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                // Only draw orbits for planets that are actually orbiting this star
                if distance < 80.0 && distance > 8.0 {
                    let orbit_radius = distance * zoom;
                    
                    // Orbit color based on distance from star
                    let orbit_color = if distance < 30.0 {
                        // Close orbits - bright blue
                        Color::new(0.3, 0.6, 1.0, base_alpha * 1.5)
                    } else if distance < 50.0 {
                        // Medium orbits - regular blue
                        Color::new(0.2, 0.5, 0.9, base_alpha * 1.2)
                    } else {
                        // Far orbits - dark blue
                        Color::new(0.1, 0.4, 0.8, base_alpha)
                    };
                    
                    // Line thickness based on planet mass
                    let line_thickness = if planet.mass > 500.0 {
                        base_line_thickness * 1.3
                    } else if planet.mass > 200.0 {
                        base_line_thickness * 1.1
                    } else {
                        base_line_thickness
                    };
                    
                    // Draw the main orbit circle
                    draw_circle_lines(
                        star_x, star_y, 
                        orbit_radius, 
                        line_thickness,
                        orbit_color
                    );
                    
                    // Add a subtle inner ring (optional)
                    if zoom > 2.5 && line_thickness > 0.5 {
                        let inner_orbit_color = Color::new(
                            orbit_color.r * 1.2,
                            orbit_color.g * 1.2,
                            orbit_color.b * 1.2,
                            orbit_color.a * 0.7
                        );
                        
                        draw_circle_lines(
                            star_x, star_y,
                            orbit_radius * 0.99,
                            line_thickness * 0.7,
                            inner_orbit_color
                        );
                    }
                    
                    // Add dots on orbit for better visibility when really zoomed in
                    if zoom > 4.0 && orbit_radius > 20.0 {
                        let point_count = 12;
                        for i in 0..point_count {
                            let angle = i as f32 * 2.0 * std::f32::consts::PI / point_count as f32;
                            let point_x = star_x + orbit_radius * angle.cos();
                            let point_y = star_y + orbit_radius * angle.sin();
                            
                            draw_circle(
                                point_x, point_y,
                                1.5,
                                Color::new(0.4, 0.6, 1.0, 0.6)
                            );
                        }
                    }
                }
            }
            
            // Add galactic orbits for stars far from center
            if zoom > 1.0 {
                let dx = star.x;
                let dy = star.y;
                let distance_from_center = (dx * dx + dy * dy).sqrt();
                
                // If star is far from galactic center, show its galactic orbit
                if distance_from_center > 200.0 && distance_from_center < 800.0 {
                    let galactic_orbit_radius = distance_from_center * zoom;
                    let galactic_color = Color::new(0.5, 0.3, 0.8, 0.08);  // Faint purple
                    
                    draw_circle_lines(
                        center_x, center_y,
                        galactic_orbit_radius,
                        0.3,
                        galactic_color
                    );
                }
            }
        }
        
        // Add binary star orbits
        if zoom > 2.0 {
            self.draw_binary_orbits(objects, center_x, center_y, zoom);
        }
    }
    
    // Draw orbits for binary star systems
    fn draw_binary_orbits(&self, objects: &[CelestialObject], center_x: f32, center_y: f32, zoom: f32) {
        let stars: Vec<_> = objects.iter()
            .filter(|obj| matches!(obj.object_type, ObjectType::Star | 
                ObjectType::WhiteDwarf | 
                ObjectType::NeutronStar))
            .collect();
        
        for i in 0..stars.len() {
            for j in (i + 1)..stars.len() {
                let star1 = stars[i];
                let star2 = stars[j];
                
                let dx = star2.x - star1.x;
                let dy = star2.y - star1.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                // If stars are close together, draw a binary orbit
                if distance < 40.0 && distance > 10.0 {
                    let center_x_world = (star1.x + star2.x) / 2.0;
                    let center_y_world = (star1.y + star2.y) / 2.0;
                    
                    let screen_center_x = center_x + center_x_world * zoom;
                    let screen_center_y = center_y + center_y_world * zoom;
                    
                    let orbit_radius = distance * zoom / 2.0;
                    
                    // Binary orbit - green
                    draw_circle_lines(
                        screen_center_x, screen_center_y,
                        orbit_radius,
                        0.4,
                        Color::new(0.4, 0.8, 0.4, 0.2)
                    );
                    
                    // Connecting line between stars
                    let star1_x = center_x + star1.x * zoom;
                    let star1_y = center_y + star1.y * zoom;
                    let star2_x = center_x + star2.x * zoom;
                    let star2_y = center_y + star2.y * zoom;
                    
                    draw_line(
                        star1_x, star1_y,
                        star2_x, star2_y,
                        0.3,
                        Color::new(0.4, 0.8, 0.4, 0.15)
                    );
                }
            }
        }
    }
    
    // Draw a single celestial object
    fn draw_stable_object(&self, obj: &CelestialObject, center_x: f32, center_y: f32, zoom: f32) {
        let screen_x = center_x + obj.x * zoom;
        let screen_y = center_y + obj.y * zoom;
        let base_radius = obj.radius * zoom;
        let display_radius = base_radius.max(0.3).min(50.0);  // Don't get too big or small
        
        let color = self.get_object_color(obj);
        
        match obj.object_type {
            ObjectType::Star => {
                draw_circle(screen_x, screen_y, display_radius, color);
                
                // Add a subtle glow for larger stars
                if display_radius > 2.0 {
                    let glow_color = Color::new(color.r, color.g, color.b, 0.2);
                    draw_circle(screen_x, screen_y, display_radius * 1.3, glow_color);
                }
            }
            ObjectType::Planet => {
                draw_circle(screen_x, screen_y, display_radius, color);
                
                // Add a ring for larger planets
                if display_radius > 1.5 && obj.mass > 200.0 {
                    draw_circle_lines(screen_x, screen_y, display_radius * 1.2, 0.5,
                        Color::new(color.r * 0.8, color.g * 0.8, color.b * 0.8, 0.4));
                }
            }
            ObjectType::GalaxyCenter => {
                draw_circle(screen_x, screen_y, display_radius, color);
                
                // Galactic centers get extra glow
                let glow_color = Color::new(color.r, color.g, color.b, 0.15);
                draw_circle(screen_x, screen_y, display_radius * 1.5, glow_color);
            }
            ObjectType::BlackHole => {
                let event_horizon = (obj.event_horizon_radius * zoom).max(1.2);
                draw_circle(screen_x, screen_y, event_horizon, BLACK);
                
                // Add accretion disk for larger black holes
                if event_horizon > 2.0 {
                    let disk_color = Color::new(0.5, 0.3, 0.6, 0.2);
                    draw_circle(screen_x, screen_y, event_horizon * 2.0, disk_color);
                }
            }
            ObjectType::DarkMatter => {
                // Dark matter is mysterious - just show an outline
                if display_radius > 1.0 {
                    draw_circle_lines(screen_x, screen_y, display_radius, 0.5, color);
                }
            }
            ObjectType::DarkEnergy => {
                // Dark energy is even more mysterious - tiny dots
                if display_radius > 0.5 {
                    draw_circle(screen_x, screen_y, display_radius * 0.3, color);
                }
            }
            ObjectType::Comet => {
                draw_circle(screen_x, screen_y, display_radius, color);
            }
            ObjectType::Asteroid => {
                draw_circle(screen_x, screen_y, display_radius, color);
            }
            ObjectType::WhiteDwarf => {
                draw_circle(screen_x, screen_y, display_radius, color);
            }
            ObjectType::NeutronStar => {
                draw_circle(screen_x, screen_y, display_radius, color);
            }
            ObjectType::Pulsar => {
                draw_circle(screen_x, screen_y, display_radius, color);
            }
        }
    }
    //----------------------------------
    // Draw faint lines between close objects
    fn draw_object_connections(&self, objects: &[CelestialObject], center_x: f32, center_y: f32, zoom: f32) {
        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                let obj1 = &objects[i];
                let obj2 = &objects[j];
                
                let dx = obj2.x - obj1.x;
                let dy = obj2.y - obj1.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                // Only connect objects that are somewhat close
                if distance < 30.0 && distance > 5.0 {
                    let x1 = center_x + obj1.x * zoom;
                    let y1 = center_y + obj1.y * zoom;
                    let x2 = center_x + obj2.x * zoom;
                    let y2 = center_y + obj2.y * zoom;
                    
                    let strength = 1.0 / distance;
                    let alpha = (strength * 0.2).min(0.1);
                    
                    draw_line(x1, y1, x2, y2, 0.3,
                        Color::new(0.3, 0.5, 0.8, alpha));
                }
            }
        }
    }
    
    // get the display color 
    fn get_object_color(&self, obj: &CelestialObject) -> Color {
        match obj.object_type {
            ObjectType::Star => {
                let r = obj.color.0 as f32 / 255.0;
                let g = obj.color.1 as f32 / 255.0;
                let b = obj.color.2 as f32 / 255.0;
                Color::new(r, g, b, 1.0)
            }
            ObjectType::Planet => {
                let r = obj.color.0 as f32 / 255.0;
                let g = obj.color.1 as f32 / 255.0;
                let b = obj.color.2 as f32 / 255.0;
                Color::new(r, g, b, 1.0)
            }
            ObjectType::GalaxyCenter => Color::new(1.0, 1.0, 0.7, 1.0),  // Yellowish
            ObjectType::DarkMatter => Color::new(0.3, 0.3, 0.6, 0.2),    // Dark purple, faint
            ObjectType::DarkEnergy => Color::new(0.6, 0.2, 0.7, 0.1),    // Purple, very faint
            ObjectType::BlackHole => Color::new(0.0, 0.0, 0.0, 1.0),     // Well, black
            ObjectType::WhiteDwarf => Color::new(0.9, 0.9, 1.0, 1.0),    // White-blue
            ObjectType::NeutronStar => Color::new(0.8, 0.8, 1.0, 1.0),   // Blue-white
            ObjectType::Pulsar => Color::new(0.7, 0.8, 1.0, 1.0),        // Blue with hint of white
            ObjectType::Comet => Color::new(0.7, 0.8, 1.0, 1.0),         // Icy blue
            ObjectType::Asteroid => Color::new(0.5, 0.5, 0.5, 1.0),      // Gray rock
        }
    }
    
    pub fn draw_object_info_with_button(&self, info: &str, screen_width: f32) {
        // Dont show on very small screens
        if screen_width < 400.0 {
            return;
        }
        
        let box_x = 20.0;
        let box_y = 20.0;
        let box_width = 350.0_f32.min(screen_width * 0.3);
        let box_height = 280.0;
        
        // Info panel background
        draw_rectangle(box_x, box_y, box_width, box_height, Color::new(0.0, 0.0, 0.0, 0.85));
        draw_rectangle_lines(box_x, box_y, box_width, box_height, 1.5, Color::new(0.6, 0.8, 1.0, 0.8));
        
        // Title
        draw_text("OBJECT INFORMATION", box_x + 10.0, box_y + 30.0, 20.0, Color::new(1.0, 1.0, 0.8, 1.0));
        
        // Info text 
        let mut current_y = box_y + 60.0;
        let lines: Vec<&str> = info.split('\n').collect();
        
        for line in lines {
            let text_height = self.draw_text_wrapped(
                line,
                box_x + 10.0,
                current_y,
                16.0,
                WHITE,
                box_width - 20.0
            );
            current_y += text_height + 5.0;
            
            // Stop if  running out of space
            if current_y > box_y + box_height - 50.0 {
                break;
            }
        }
        
        // "Learn More" button 
        let button_x = box_x + 10.0;
        let button_y = box_y + box_height - 40.0;
        let button_width = box_width - 20.0;
        let button_height = 30.0;
        
        // Button background
        draw_rectangle(button_x, button_y, button_width, button_height, Color::new(0.1, 0.5, 0.2, 0.9));
        draw_rectangle_lines(button_x, button_y, button_width, button_height, 1.5, Color::new(0.3, 0.8, 0.3, 1.0));
        
        // Button text 
        let button_text = "LEARN MORE (PRESS Tab)";
        let text_width = self.measure_text_width(button_text, 16.0);
        draw_text(
            button_text,
            button_x + (button_width - text_width) / 2.0,
            button_y + 20.0,
            16.0,
            Color::new(1.0, 1.0, 0.8, 1.0)
        );
        
        //  hint
        draw_text(
            "",
            button_x,
            button_y + 35.0,
            12.0,
            Color::new(0.8, 0.9, 1.0, 0.7)
        );
    }
    
    // Draw detailed information modal
    pub fn draw_detail_modal(&self, obj: &CelestialObject, show_modal: &mut bool, screen_width: f32, screen_height: f32) {
        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_width, screen_height, Color::new(0.0, 0.0, 0.0, 0.7));
        
        // Responsive
        let modal_width = 700.0_f32.min(screen_width * 0.85).max(300.0);
        let modal_height = 500.0_f32.min(screen_height * 0.85).max(300.0);
        let modal_x = (screen_width - modal_width) / 2.0;
        let modal_y = (screen_height - modal_height) / 2.0;
        
        // background
        draw_rectangle(modal_x, modal_y, modal_width, modal_height, Color::new(0.05, 0.05, 0.1, 0.95));
        draw_rectangle_lines(modal_x, modal_y, modal_width, modal_height, 2.0, Color::new(0.6, 0.8, 1.0, 0.9));
        
        // Title
        let title = match obj.object_type {
            ObjectType::Star => "STAR - Detailed Information",
            ObjectType::Planet => "PLANET - Detailed Information",
            ObjectType::GalaxyCenter => "GALAXY CENTER - Detailed Information",
            ObjectType::DarkMatter => "DARK MATTER - Detailed Information",
            ObjectType::DarkEnergy => "DARK ENERGY - Detailed Information",
            ObjectType::BlackHole => "BLACK HOLE - Detailed Information",
            ObjectType::NeutronStar => "NEUTRON STAR - Detailed Information",
            ObjectType::Pulsar => "PULSAR - Detailed Information",
            ObjectType::WhiteDwarf => "WHITE DWARF - Detailed Information",
            ObjectType::Comet => "COMET - Detailed Information",
            ObjectType::Asteroid => "ASTEROID - Detailed Information",
        };
        
        // Draw centered title
        let title_font_size = if modal_width < 500.0 { 22.0 } else { 26.0 };
        let title_width = self.measure_text_width(title, title_font_size);
        draw_text(
            title,
            modal_x + (modal_width - title_width) / 2.0,
            modal_y + 40.0,
            title_font_size,
            Color::new(1.0, 1.0, 0.6, 1.0)
        );
        
        // Get and drw detailed description
        let description = self.get_detailed_description(obj);
        let font_size = if modal_width < 500.0 { 16.0 } else { 18.0 };
        
        // Draw wrappeddescription
        self.draw_text_wrapped(
            &description,
            modal_x + 20.0,
            modal_y + 80.0,
            font_size,
            WHITE,
            modal_width - 40.0
        );
        
        // Close button
        let close_button_y = modal_y + modal_height - 50.0;
        let close_button_width = 100.0_f32.min(modal_width * 0.3);
        let close_button_x = modal_x + (modal_width - close_button_width) / 2.0;
        let close_button_height = 35.0;
        
        // Close button background
        draw_rectangle(close_button_x, close_button_y, close_button_width, close_button_height, 
                      Color::new(0.6, 0.2, 0.2, 0.9));
        draw_rectangle_lines(close_button_x, close_button_y, close_button_width, close_button_height, 
                           1.5, Color::new(1.0, 0.4, 0.4, 1.0));
        
        // Close button text
        let close_text = "CLOSE";
        let close_text_width = self.measure_text_width(close_text, 18.0);
        draw_text(
            close_text,
            close_button_x + (close_button_width - close_text_width) / 2.0,
            close_button_y + 22.0,
            18.0,
            Color::new(1.0, 1.0, 0.8, 1.0)
        );
        
        // Hover effect for close button
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= close_button_x && mouse_pos.0 <= close_button_x + close_button_width &&
           mouse_pos.1 >= close_button_y && mouse_pos.1 <= close_button_y + close_button_height {
            draw_rectangle(close_button_x, close_button_y, close_button_width, close_button_height, 
                          Color::new(0.8, 0.3, 0.3, 0.5));
            
            // Close on click
            if is_mouse_button_pressed(MouseButton::Left) {
                *show_modal = false;
            }
        }
    }
    
    // Get detailed scientific description for each object type
    fn get_detailed_description(&self, obj: &CelestialObject) -> String {
        match obj.object_type {
            ObjectType::Star => format!(
                "STARS are massive, luminous spheres of plasma held together by gravity.\n\n\
                This star has a mass of {:.1} solar masses and a temperature of {:.0} K.\n\
                Stars are the fundamental building blocks of galaxies and are responsible\n\
                for the synthesis of most elements in the universe through nuclear fusion.\n\n\
                Main sequence stars like this one convert hydrogen into helium in their cores,\n\
                releasing enormous amounts of energy. The color indicates its surface temperature:\n\
                blue stars are hottest (>10,000 K), white/yellow stars like our Sun are medium\n\
                (5,000-6,000 K), and red stars are coolest (3,000-4,000 K).\n\n\
                This star will eventually exhaust its nuclear fuel and evolve into a\n\
                red giant, then a planetary nebula, leaving behind a white dwarf remnant.",
                obj.mass / 2e5, obj.temperature
            ),
            
            ObjectType::Planet => format!(
                "PLANETS are celestial bodies that orbit stars, are massive enough to be\n\
                rounded by their own gravity, and have cleared their orbital region of debris.\n\n\
                This planet has a mass of {:.1} Earth masses and orbits its star.\n\n\
                Planets can be terrestrial (rocky, like Earth) or gas giants (like Jupiter).\n\
                Based on its temperature ({:.0} K), this planet appears to be {}\n\n\
                Planetary formation occurs in protoplanetary disks around young stars,\n\
                where dust and gas coalesce through accretion. Planets play crucial roles\n\
                in their systems, influencing asteroid distribution and potentially\n\
                supporting life through stable orbital conditions.",
                obj.mass / 100.0,
                obj.temperature,
                if obj.temperature > 400.0 { "a gas giant." } else { "a terrestrial planet." }
            ),
            
            ObjectType::BlackHole => format!(
                "BLACK HOLES are regions of spacetime where gravity is so strong that\n\
                nothing, not even light, can escape.\n\n\
                This black hole has a mass of {:.1} solar masses with an event horizon\n\
                radius of {:.2} km.\n\n\
                Black holes form when massive stars collapse at the end of their life\n\
                cycles or through the merger of other compact objects. They are characterized\n\
                by their event horizon - the point of no return.\n\n\
                This appears to be a {} black hole. Black holes warp spacetime,\n\
                cause gravitational lensing, and can have accretion disks of hot matter\n\
                spiraling into them.",
                obj.mass / 2e5,
                obj.event_horizon_radius / 1000.0,
                if obj.mass > 1e6 { "supermassive" } else { "stellar-mass" }
            ),
            
            ObjectType::NeutronStar => format!(
                "NEUTRON STARS are the collapsed cores of massive stars that have\n\
                undergone supernova explosions.\n\n\
                This neutron star has a mass of {:.1} solar masses compressed into\n\
                a sphere only {:.1} km in radius. It rotates every {:.3} seconds.\n\n\
                Neutron stars are incredibly dense - one teaspoon would weigh billions\n\
                of tons. They are supported against further collapse by neutron degeneracy\n\
                pressure. Some neutron stars emit beams of radiation, becoming pulsars.",
                obj.mass / 2e5,
                obj.radius,
                obj.rotation_period
            ),
            
            ObjectType::Pulsar => format!(
                "PULSARS are highly magnetized, rotating neutron stars that emit beams\n\
                of electromagnetic radiation from their magnetic poles.\n\n\
                This pulsar rotates every {:.3} seconds.\n\n\
                Pulsars are often called 'cosmic lighthouses' because their beams sweep\n\
                across space like lighthouse beams. When aligned with Earth, we detect\n\
                regular pulses of radiation.\n\n\
                They are formed when massive stars explode as supernovae, leaving behind\n\
                rapidly spinning, magnetized cores.",
                obj.rotation_period
            ),
            
            ObjectType::WhiteDwarf => format!(
                "WHITE DWARFS are the stellar remnants left after low and medium mass stars\n\
                exhaust their nuclear fuel.\n\n\
                This white dwarf has a mass of {:.1} solar masses packed into a sphere\n\
                only {:.1} Earth radii in size, with a temperature of {:.0} K.\n\n\
                White dwarfs are supported by electron degeneracy pressure and have\n\
                densities of about 1 ton per cubic centimeter. They slowly cool over\n\
                billions of years.",
                obj.mass / 2e5,
                obj.radius / 0.5,
                obj.temperature
            ),
            
            ObjectType::Comet => format!(
                "COMETS are icy small Solar System bodies that, when passing close to the Sun,\n\
                warm and begin to release gases, producing a visible atmosphere or coma.\n\n\
                This comet is traveling at {:.1} km/s with a temperature of {:.0} K.\n\n\
                Comets are composed of frozen gases, rock, and dust. They originate\n\
                from the Kuiper Belt and Oort Cloud in the outer Solar System.\n\
                When heated by the Sun, comets develop spectacular tails.",
                (obj.vx * obj.vx + obj.vy * obj.vy).sqrt() / 1000.0,
                obj.temperature
            ),
            
            ObjectType::Asteroid => format!(
                "ASTEROIDS are minor planets of the inner Solar System, ranging in size\n\
                from about 1 meter to hundreds of kilometers.\n\n\
                This asteroid has a mass of {:.1} tons and is traveling at {:.1} km/s.\n\n\
                Most asteroids are found in the asteroid belt between Mars and Jupiter.\n\
                They are remnants from the Solar System's formation that never coalesced into planets.",
                obj.mass,
                (obj.vx * obj.vx + obj.vy * obj.vy).sqrt() / 1000.0
            ),
            
            ObjectType::GalaxyCenter => format!(
                "GALAXY CENTERS are the central regions of galaxies, typically containing\n\
                high stellar densities and often hosting supermassive black holes.\n\n\
                This galactic center has a mass of {:.1} solar masses.\n\n\
                The centers of galaxies are sites of intense activity, with stars\n\
                moving at high velocities and often showing evidence of past mergers.",
                obj.mass / 2e5
            ),
            
            ObjectType::DarkMatter => format!(
                "DARK MATTER is a hypothetical form of matter that is thought to account\n\
                for approximately 85% of the matter in the universe.\n\n\
                This dark matter clump has a mass of {:.1} solar masses but interacts\n\
                only through gravity.\n\n\
                Dark matter does not emit, absorb, or reflect light, making it invisible\n\
                to electromagnetic observations.",
                obj.mass / 2e5
            ),
            
            ObjectType::DarkEnergy => format!(
                "DARK ENERGY is a mysterious form of energy that permeates all of space\n\
                and tends to accelerate the expansion of the universe.\n\n\
                This region shows dark energy effects with density parameter {:.3}.\n\n\
                Dark energy is the dominant component of the universe (about 68% of\n\
                the total energy density). Its existence was inferred from observations\n\
                that the universe's expansion is accelerating.",
                obj.mass / 1e6
            ),
        }
    }
    
    // Draw controls/help panel
    pub fn draw_help(&self, expansion_speed: f32, universe_age: f64, zoom: f32, 
        offset_x: f32, offset_y: f32, time_scale: &str, 
        fullscreen: bool, screen_width: f32, screen_height: f32) {

        // Mini help for tiny screens
        if screen_width < 600.0 {
            self.draw_minimal_help(expansion_speed, universe_age, zoom, time_scale, fullscreen, screen_width);
            return;
        }

        let speed_text = match expansion_speed {
            x if x < 0.1 => "Minimal",
            x if x < 0.2 => "Slow",
            _ => "Medium",
        };

        let age_display = if universe_age < 1.0 {
            format!("{:.2} years", universe_age * 1e6)
        } else {
            format!("{:.1} Myr", universe_age)
        };

        // Help text (including F5)
        let help_lines = [
            "CONTROLS",
            &format!("Expansion: {}", speed_text),
            "1: Minimal  2: Slow  3: Medium",
            &format!("Time: {} (Space: Pause)", time_scale),
            "Shift: Slow Motion",
            "Zoom: Mouse Wheel",
            "Drag Camera: Right Click",
            "Reset: R or Middle Click",
            "Select: Left Click",
            "Details: Tab (when object selected)",
            "Close Modal: ESC or Click Outside",
            &format!("Fullscreen: F (Now: {})", if fullscreen { "ON" } else { "OFF" }),
            &format!("Age: {}", age_display),
            &format!("Zoom: {:.1}x", zoom),
        ];

        let start_x = screen_width - 360.0;
        let mut start_y = 25.0;

        // Start lower on short screens
        if screen_height < 600.0 {
            start_y = screen_height - (help_lines.len() as f32 * 22.0) - 10.0;
        }

        for (i, text) in help_lines.iter().enumerate() {
            let color = if i == 0 {
                Color::new(1.0, 1.0, 0.6, 1.0)  // Yellow title
            } else if i == 1 || i == 3 {
                Color::new(0.6, 1.0, 0.6, 1.0)  // Green for important info
            } else if i == 9 {  // Different color for F5 line
                Color::new(0.8, 1.0, 0.6, 1.0)  // Bright green
            } else if i >= 8 {
                Color::new(0.8, 0.9, 1.0, 1.0)  // Light blue for controls
            } else {
                WHITE
            };

            let font_size = if i == 0 { 20 } else { 16 };

            draw_text(
                text,
                start_x,
                start_y + (i as f32 * 22.0),
                font_size as f32,
                color,
            );
        }
    }

    // Mini help for small screens
    fn draw_minimal_help(&self, expansion_speed: f32, universe_age: f64, zoom: f32, 
                time_scale: &str, fullscreen: bool, screen_width: f32) {
        let essential_lines = [
            "CONTROLS (Minimal)",
            &format!("Exp: {} | Time: {}", 
                match expansion_speed {
                    x if x < 0.1 => "Min",
                    x if x < 0.2 => "Slow",
                    _ => "Med",
                },
                time_scale
            ),
            "Zoom: Wheel | Drag: Right",
            "Select: Left | Reset: R",
            "Details: Tab",
            "Fullscreen: F",
            &format!("Age: {:.1}M | Zoom: {:.1}x", universe_age, zoom),
        ];

        let start_x = screen_width - 200.0;
        let start_y = 10.0;

        for (i, text) in essential_lines.iter().enumerate() {
            let font_size = if i == 0 { 14.0 } else { 12.0 };
            let color = if i == 4 {  // Different color for F5
                Color::new(0.8, 1.0, 0.6, 1.0)
            } else if i == 0 {
                Color::new(1.0, 1.0, 0.6, 1.0)
            } else {
                Color::new(0.8, 0.9, 1.0, 1.0)
            };

            draw_text(
                text,
                start_x,
                start_y + (i as f32 * 18.0),
                font_size,
                color,
            );
        }
    }
}