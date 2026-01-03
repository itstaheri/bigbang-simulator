use macroquad::prelude::*;

mod universe;
mod physics;
mod objects;
mod gravity;
mod expansion;
mod rendering;

use universe::Universe;
use rendering::Renderer;

#[macroquad::main("Big Bang Simulator")]
async fn main() {
    let mut current_width = screen_width();
    let mut current_height = screen_height();
    
    let mut universe = Universe::new(current_width, current_height);
    let mut renderer = Renderer::new();
    
    let mut expansion_speed = 0.1;
    let mut zoom = 1.0;
    let mut camera_offset_x = 0.0;
    let mut camera_offset_y = 0.0;
    let mut is_dragging = false;
    let mut drag_start_pos = (0.0, 0.0);
    
    let mut show_info = false;
    let mut selected_object_info: Option<String> = None;
    let mut selected_object_index: Option<usize> = None;
    let mut show_detail_modal = false;
    let mut fullscreen = false;
    
    // Main loop
    loop {
        // Update window dimensions if resized
        current_width = screen_width();
        current_height = screen_height();
        
        // Cosmic speed 
        if is_key_pressed(KeyCode::Key1) {
            expansion_speed = 0.05;
        }
        if is_key_pressed(KeyCode::Key2) {
            expansion_speed = 0.15;
        }
        if is_key_pressed(KeyCode::Key3) {
            expansion_speed = 0.3;
        }
        
        // Toggle fullscreen
        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
            next_frame().await;
            continue;
        }
        
        if is_key_pressed(KeyCode::Tab) && show_info && selected_object_index.is_some() {
            show_detail_modal = true;
        }
        
        // Zoom in/out with mouse wheel
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            let mouse_pos = mouse_position();
            // Convert mouse position to world space for smooth zoom
            let mouse_world_x = (mouse_pos.0 - current_width / 2.0 - camera_offset_x) / zoom;
            let mouse_world_y = (mouse_pos.1 - current_height / 2.0 - camera_offset_y) / zoom;
            
            let old_zoom = zoom;
            zoom *= 1.0 + wheel * 0.1;
            zoom = zoom.clamp(0.05, 30.0); // Dont zoom too far!
            
            // Keep mouse position stable during zoom
            camera_offset_x += mouse_world_x * (old_zoom - zoom);
            camera_offset_y += mouse_world_y * (old_zoom - zoom);
        }
        
        // Right click drag to pan around
        if is_mouse_button_down(MouseButton::Right) {
            let mouse_pos = mouse_position();
            
            if !is_dragging {
                is_dragging = true;
                drag_start_pos = mouse_pos;
            } else {
                let dx = mouse_pos.0 - drag_start_pos.0;
                let dy = mouse_pos.1 - drag_start_pos.1;
                camera_offset_x += dx * 0.5;
                camera_offset_y += dy * 0.5;
                drag_start_pos = mouse_pos;
            }
        } else {
            is_dragging = false;
        }
        
        // Reset view with middle click or R key
        if is_mouse_button_pressed(MouseButton::Middle) || is_key_pressed(KeyCode::R) {
            camera_offset_x = 0.0;
            camera_offset_y = 0.0;
            zoom = 1.0;
        }
        
        // Time manipulation controls
        let mut time_scale = 1.0;
        if is_key_down(KeyCode::Space) {
            time_scale = 0.0; // Pause the universe!
        } else if is_key_down(KeyCode::LeftShift) {
            time_scale = 0.3; // Slow-mo cosmic ballet
        }
        
        // Click to select celestial objects
        if is_mouse_button_pressed(MouseButton::Left) && !is_dragging && !show_detail_modal {
            let mouse_pos = mouse_position();
            
            // Convert click to world coordinates
            let world_x = (mouse_pos.0 - current_width / 2.0 - camera_offset_x) / zoom;
            let world_y = (mouse_pos.1 - current_height / 2.0 - camera_offset_y) / zoom;
            
            // Check if we clicked on something interesting
            if let Some((obj_idx, obj)) = universe.get_object_at_position(world_x, world_y, zoom) {
                selected_object_index = Some(obj_idx);
                
                // Translate enum to human-readable string
                let object_type_str = match obj.object_type {
                    crate::objects::ObjectType::Star => "Star",
                    crate::objects::ObjectType::Planet => "Planet",
                    crate::objects::ObjectType::GalaxyCenter => "Galaxy Center",
                    crate::objects::ObjectType::DarkMatter => "Dark Matter",
                    crate::objects::ObjectType::DarkEnergy => "Dark Energy",
                    crate::objects::ObjectType::BlackHole => "Black Hole",
                    crate::objects::ObjectType::NeutronStar => "Neutron Star",
                    crate::objects::ObjectType::Pulsar => "Pulsar",
                    crate::objects::ObjectType::WhiteDwarf => "White Dwarf",
                    crate::objects::ObjectType::Comet => "Comet",
                    crate::objects::ObjectType::Asteroid => "Asteroid",
                };
                
                // Extra info for special objects
                let extra_info = match obj.object_type {
                    crate::objects::ObjectType::NeutronStar => {
                        format!("\nRotation: {:.3}s\nMag Field: {:.0e} T", 
                            obj.rotation_period, obj.magnetic_field)
                    }
                    crate::objects::ObjectType::Pulsar => {
                        format!("\nRotation: {:.3}s\nMag Field: {:.0e} T\nBeam Period: {:.2}s", 
                            obj.rotation_period, obj.magnetic_field, 1.0/obj.rotation_period)
                    }
                    crate::objects::ObjectType::WhiteDwarf => {
                        "\nType: Degenerate Star\nDensity: ~1 ton/cm³".to_string()
                    }
                    crate::objects::ObjectType::BlackHole => {
                        format!("\nEvent Horizon: {:.2} km", obj.event_horizon_radius / 1000.0)
                    }
                    _ => "".to_string(),
                };
                
                // Build info string
                selected_object_info = Some(format!(
                    "Type: {}\nMass: {:.2e} kg\nRadius: {:.1} km\nSpeed: {:.2} km/s\nTemperature: {:.0} K\nAge: {:.1} Myr\nPosition: ({:.0}, {:.0}) km{}",
                    object_type_str,
                    obj.mass,
                    obj.radius / 1000.0,
                    (obj.vx * obj.vx + obj.vy * obj.vy).sqrt() / 1000.0,
                    obj.temperature,
                    obj.get_age(universe.age) / (1e6 * 365.0 * 24.0 * 3600.0),
                    obj.x / 1000.0, obj.y / 1000.0,
                    extra_info
                ));
                show_info = true;
                show_detail_modal = false;
            } else {
                //  clear selection
                selected_object_info = None;
                selected_object_index = None;
                show_info = false;
                show_detail_modal = false;
            }
        }
        
        // Close modal with Escape key
        if is_key_pressed(KeyCode::Escape) {
            show_detail_modal = false;
        }
        
        // Close modal by clicking outside
        if show_detail_modal && is_mouse_button_pressed(MouseButton::Left) {
            let mouse_pos = mouse_position();
            let modal_width = 700.0_f32.min(current_width * 0.8);
            let modal_height = 500.0_f32.min(current_height * 0.8);
            let modal_x = (current_width - modal_width) / 2.0;
            let modal_y = (current_height - modal_height) / 2.0;
            
            if mouse_pos.0 < modal_x || mouse_pos.0 > modal_x + modal_width ||
               mouse_pos.1 < modal_y || mouse_pos.1 > modal_y + modal_height {
                show_detail_modal = false;
            }
        }
        
        // Update the universe 
        universe.update(expansion_speed * time_scale);
        
        clear_background(Color::new(0.02, 0.02, 0.05, 1.0));
        
        renderer.draw_universe(&universe, zoom, camera_offset_x, camera_offset_y, current_width, current_height);
        
        // Show object info panel
        if show_info && !show_detail_modal {
            if let Some(ref info) = selected_object_info {
                renderer.draw_object_info_with_button(info, current_width);
            }
        }
        
        // Show detailed modal view
        if show_detail_modal && selected_object_index.is_some() {
            if let Some(obj_idx) = selected_object_index {
                if let Some(obj) = universe.get_object_by_index(obj_idx) {
                    renderer.draw_detail_modal(obj, &mut show_detail_modal, current_width, current_height);
                }
            }
        }
        
        //  scale indicator
        let time_scale_text = match time_scale {
            0.0 => "PAUSED",
            x if x < 0.5 => "SLOW",
            _ => "NORMAL",
        };
        
        // When modal is open
        if !show_detail_modal {
            renderer.draw_help(expansion_speed, universe.age, zoom, camera_offset_x, camera_offset_y, 
                             time_scale_text, fullscreen, current_width, current_height);
        }
        
        // Wait for next frame
        next_frame().await;
    }
}