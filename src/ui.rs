use macroquad::prelude::*;
use crate::objects::CelestialObject;

pub struct UIStyle {
    pub panel_bg: Color,
    
    pub border_color: Color,
    
    pub text_color: Color,
    
    pub highlight_color: Color,
    
    pub button_bg: Color,
    pub button_hover: Color,
    pub button_text: Color,
}

impl UIStyle {
    pub fn cosmic_dark() -> Self {
        UIStyle {
            panel_bg: Color::new(0.05, 0.05, 0.1, 0.9),
            border_color: Color::new(0.3, 0.6, 0.9, 0.8),
            text_color: WHITE,
            highlight_color: Color::new(1.0, 0.8, 0.2, 1.0),
            button_bg: Color::new(0.1, 0.3, 0.5, 0.9),
            button_hover: Color::new(0.2, 0.4, 0.7, 0.9),
            button_text: Color::new(1.0, 1.0, 0.8, 1.0),
        }
    }
    
    pub fn nebula() -> Self {
        UIStyle {
            panel_bg: Color::new(0.08, 0.05, 0.15, 0.9),
            border_color: Color::new(0.6, 0.3, 0.8, 0.8),
            text_color: Color::new(0.9, 0.9, 1.0, 1.0),
            highlight_color: Color::new(0.8, 0.4, 1.0, 1.0),
            button_bg: Color::new(0.3, 0.1, 0.5, 0.9),
            button_hover: Color::new(0.5, 0.2, 0.8, 0.9),
            button_text: Color::new(1.0, 0.9, 1.0, 1.0),
        }
    }
}

pub struct SelectionState {
    pub selected_object_index: Option<usize>,
    pub selected_object_info: Option<String>,
    pub selection_time: std::time::Instant,
    pub selection_pulse: f32,
}

pub struct UI {
    style: UIStyle,
    time: f32,
}

impl UI {
    pub fn new(style: UIStyle) -> Self {
        UI {
            style,
            time: 0.0,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
    }
    
    pub fn draw_control_panel(
        &self,
        controls: &crate::SimulationControls,
        camera: &crate::CameraState,
        fps: f32,
        screen_size: &Vec2,
    ) {
        let speed_text = match controls.expansion_speed {
            x if x < 0.1 => "Minimal",
            x if x < 0.2 => "Slow",
            _ => "Medium",
        };
        
        let time_state = if controls.is_paused {
            "⏸️ PAUSED"
        } else if controls.time_scale < 0.5 {
            "🐌 SLOW"
        } else {
            "▶️ NORMAL"
        };
        
        let panel_width = 350.0_f32.min(screen_size.x * 0.25);
        let panel_x = screen_size.x - panel_width - 20.0;
        let panel_y = 20.0;
        let panel_height = 300.0;
        
        self.draw_rounded_panel(panel_x, panel_y, panel_width, panel_height);
        
        self.draw_text_shadowed(
            "COSMIC CONTROLS",
            panel_x + panel_width / 2.0,
            panel_y + 30.0,
            24.0,
            self.style.highlight_color,
            true
        );
        
        let mut y_offset = panel_y + 70.0;
        let line_height = 24.0;
        
        let info_lines = [
            format!("🌌 Expansion: {}", speed_text),
            format!("⏱️ Time: {}", time_state),
            format!("🔭 Zoom: {:.1}x", camera.zoom),
            format!("🎯 FPS: {:.0}", fps),
            format!("{}", if controls.fullscreen { "🖥️ Fullscreen: ON" } else { "🖥️ Fullscreen: OFF" }),
        ];
        
        for line in &info_lines {
            self.draw_text(line, panel_x + 15.0, y_offset, 18.0, self.style.text_color);
            y_offset += line_height;
        }
        
        y_offset += 10.0;
        self.draw_speed_buttons(controls, panel_x + 15.0, y_offset, panel_width - 30.0);
        
        y_offset += 60.0;
        self.draw_text("🖱️ Controls:", panel_x + 15.0, y_offset, 16.0, self.style.highlight_color);
        y_offset += 20.0;
        
        let control_lines = [
            "Wheel - Zoom",
            "Right Drag - Pan",
            "Left Click - Select",
            "Space - Pause",
            "R - Reset View",
            "F - Fullscreen",
        ];
        
        for line in &control_lines {
            self.draw_text(line, panel_x + 25.0, y_offset, 14.0, Color::new(0.8, 0.9, 1.0, 0.8));
            y_offset += 18.0;
        }
    }
    
    pub fn draw_object_info(&self, info: &str, controls: &crate::SimulationControls, screen_size: &Vec2) {
        let panel_width = 350.0_f32.min(screen_size.x * 0.3);
        let panel_x = 20.0;
        let panel_y = 20.0;
        let panel_height = 280.0;
        
        self.draw_rounded_panel(panel_x, panel_y, panel_width, panel_height);
        
        self.draw_text_shadowed(
            "CELESTIAL OBJECT",
            panel_x + panel_width / 2.0,
            panel_y + 30.0,
            22.0,
            self.style.highlight_color,
            true
        );
        
        let lines: Vec<&str> = info.split('\n').collect();
        let mut y_offset = panel_y + 70.0;
        
        for line in lines {
            self.draw_text_wrapped(line, panel_x + 15.0, y_offset, 16.0, self.style.text_color, panel_width - 30.0);
            y_offset += 22.0;
            
            if y_offset > panel_y + panel_height - 60.0 {
                break;
            }
        }
        
        let button_y = panel_y + panel_height - 45.0;
        let button_height = 35.0;
        
        let button_hovered = self.draw_button(
            "📖 LEARN MORE (Press Tab)",
            panel_x + 15.0,
            button_y,
            panel_width - 30.0,
            button_height,
        );
        
        if button_hovered && is_mouse_button_pressed(MouseButton::Left) {
        }
    }
    
    pub fn draw_detail_modal(
        &self,
        obj: &CelestialObject,
        show_modal: &mut bool,
        screen_size: &Vec2,
    ) {
        draw_rectangle(0.0, 0.0, screen_size.x, screen_size.y, Color::new(0.0, 0.0, 0.0, 0.7));
        
        let modal_width = 700.0_f32.min(screen_size.x * 0.85).max(300.0);
        let modal_height = 500.0_f32.min(screen_size.y * 0.85).max(300.0);
        let modal_x = (screen_size.x - modal_width) / 2.0;
        let modal_y = (screen_size.y - modal_height) / 2.0;
        
        self.draw_modal_background(modal_x, modal_y, modal_width, modal_height);
        
        let title = match obj.object_type {
            crate::objects::ObjectType::Star => "🌟 STAR - Cosmic Furnace",
            crate::objects::ObjectType::Planet => "🪐 PLANET - World in Orbit",
            crate::objects::ObjectType::GalaxyCenter => "🌌 GALACTIC CORE - Heart of a Galaxy",
            crate::objects::ObjectType::DarkMatter => "🌀 DARK MATTER - Cosmic Mystery",
            crate::objects::ObjectType::DarkEnergy => "⚡ DARK ENERGY - Accelerating Force",
            crate::objects::ObjectType::BlackHole => "🕳️ BLACK HOLE - Point of No Return",
            crate::objects::ObjectType::NeutronStar => "⭐ NEUTRON STAR - Ultra-Dense Remnant",
            crate::objects::ObjectType::Pulsar => "💫 PULSAR - Cosmic Lighthouse",
            crate::objects::ObjectType::WhiteDwarf => "💎 WHITE DWARF - Stellar Cinder",
            crate::objects::ObjectType::Comet => "☄️ COMET - Icy Wanderer",
            crate::objects::ObjectType::Asteroid => "🪨 ASTEROID - Rocky Fragment",
        };
        
        self.draw_text_shadowed(title, modal_x + modal_width / 2.0, modal_y + 40.0, 26.0, self.style.highlight_color, true);
        
        let description = self.get_detailed_description(obj);
        let font_size = if modal_width < 500.0 { 16.0 } else { 18.0 };
        
        self.draw_text_wrapped(
            &description,
            modal_x + 25.0,
            modal_y + 90.0,
            font_size,
            self.style.text_color,
            modal_width - 50.0
        );
        
        let close_button_y = modal_y + modal_height - 50.0;
        let close_button_width = 120.0;
        let close_button_x = modal_x + (modal_width - close_button_width) / 2.0;
        
        let close_hovered = self.draw_button(
            "❌ CLOSE (Esc)",
            close_button_x,
            close_button_y,
            close_button_width,
            35.0,
        );
        
        if close_hovered && is_mouse_button_pressed(MouseButton::Left) {
            *show_modal = false;
        }
    }
    
    pub fn draw_performance_overlay(&self, fps: f32, delta_time: f32, screen_size: &Vec2) {
        let x = 10.0;
        let y = screen_size.y - 100.0;
        
        draw_rectangle(x, y, 200.0, 90.0, Color::new(0.0, 0.0, 0.0, 0.7));
        
        self.draw_text(&format!("FPS: {:.0}", fps), x + 10.0, y + 25.0, 18.0, Color::new(0.6, 1.0, 0.6, 1.0));
        self.draw_text(&format!("Frame: {:.3}ms", delta_time * 1000.0), x + 10.0, y + 50.0, 16.0, WHITE);
        self.draw_text("Press P to hide", x + 10.0, y + 75.0, 14.0, Color::new(0.8, 0.8, 0.8, 0.7));
    }
    
    fn draw_rounded_panel(&self, x: f32, y: f32, width: f32, height: f32) {
        draw_rectangle(x, y, width, height, self.style.panel_bg);
        
        draw_rectangle_lines(x, y, width, height, 2.0, self.style.border_color);
        
        draw_rectangle_lines(x + 1.0, y + 1.0, width - 2.0, height - 2.0, 1.0, 
            Color::new(self.style.border_color.r, self.style.border_color.g, 
                      self.style.border_color.b, 0.3));
    }
    
    fn draw_modal_background(&self, x: f32, y: f32, width: f32, height: f32) {
        draw_rectangle(x, y, width, height, Color::new(0.07, 0.07, 0.12, 0.97));
        
        let glow_size = 10.0;
        for i in 1..=3 {
            let glow_alpha = 0.1 / i as f32;
            let glow_color = Color::new(0.3, 0.6, 0.9, glow_alpha);
            draw_rectangle_lines(
                x - glow_size * i as f32,
                y - glow_size * i as f32,
                width + glow_size * 2.0 * i as f32,
                height + glow_size * 2.0 * i as f32,
                2.0,
                glow_color
            );
        }
        
        draw_rectangle_lines(x, y, width, height, 2.0, self.style.border_color);
    }
    
    fn draw_text(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text(text, x, y, size, color);
    }
    
    fn draw_text_shadowed(&self, text: &str, x: f32, y: f32, size: f32, color: Color, centered: bool) {
        let text_width = measure_text(text, None, size as u16, 1.0).width;
        let draw_x = if centered { x - text_width / 2.0 } else { x };
        
        draw_text(text, draw_x + 1.0, y + 1.0, size, Color::new(0.0, 0.0, 0.0, 0.5));
        
        draw_text(text, draw_x, y, size, color);
    }
    
    fn draw_text_wrapped(&self, text: &str, x: f32, y: f32, size: f32, color: Color, max_width: f32) -> f32 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();
        let mut current_y = y;
        let line_height = size * 1.2;
        
        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };
            
            let test_width = measure_text(&test_line, None, size as u16, 1.0).width;
            
            if test_width <= max_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    draw_text(&current_line, x, current_y, size, color);
                    current_y += line_height;
                }
                current_line = word.to_string();
            }
        }
        
        if !current_line.is_empty() {
            draw_text(&current_line, x, current_y, size, color);
            current_y += line_height;
        }
        
        current_y - y
    }
    
    fn draw_button(&self, text: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
        let mouse_pos = mouse_position();
        let hovered = mouse_pos.0 >= x && mouse_pos.0 <= x + width &&
                      mouse_pos.1 >= y && mouse_pos.1 <= y + height;
        
        let bg_color = if hovered { self.style.button_hover } else { self.style.button_bg };
        draw_rectangle(x, y, width, height, bg_color);
        
        let border_color = if hovered { 
            Color::new(self.style.border_color.r * 1.2, 
                      self.style.border_color.g * 1.2, 
                      self.style.border_color.b * 1.2, 1.0)
        } else { 
            self.style.border_color 
        };
        draw_rectangle_lines(x, y, width, height, 2.0, border_color);
        
        let text_width = measure_text(text, None, 16, 1.0).width;
        draw_text(
            text,
            x + (width - text_width) / 2.0,
            y + height / 2.0 + 5.0,
            16.0,
            self.style.button_text
        );
        
        hovered
    }
    
    fn draw_speed_buttons(&self, controls: &crate::SimulationControls, x: f32, y: f32, width: f32) {
        let button_width = width / 3.0 - 5.0;
        let speeds = ["Minimal", "Slow", "Medium"];
        
        for i in 0..3 {
            let button_x = x + i as f32 * (button_width + 5.0);
            let active = controls.current_speed_index == i;
            
            let button_text = if active {
                format!("🔘 {}", speeds[i])
            } else {
                format!("⚪ {}", speeds[i])
            };
            
            self.draw_button(&button_text, button_x, y, button_width, 30.0);
        }
    }
    
    fn get_detailed_description(&self, obj: &CelestialObject) -> String {
        String::new()
    }
}

pub fn calculate_modal_rect(screen_size: &Vec2) -> Rect {
    let modal_width = 700.0_f32.min(screen_size.x * 0.85).max(300.0);
    let modal_height = 500.0_f32.min(screen_size.y * 0.85).max(300.0);
    let modal_x = (screen_size.x - modal_width) / 2.0;
    let modal_y = (screen_size.y - modal_height) / 2.0;
    
    Rect::new(modal_x, modal_y, modal_width, modal_height)
}
