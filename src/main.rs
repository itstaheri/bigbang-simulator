use macroquad::prelude::*;

mod camera;
mod expansion;
mod gravity;
mod objects;
mod physics;
mod rendering;
mod universe;

use camera::OrbitCamera;
use rendering::{Renderer, ViewOptions};
use universe::Universe;

fn window_conf() -> Conf {
    Conf {
        window_title: "Big Bang Simulator 3D".to_owned(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand((macroquad::miniquad::date::now() * 1_000_000.0) as u64);

    let mut universe = Universe::new(screen_width(), screen_height());
    let mut renderer = Renderer::new();
    let mut cam = OrbitCamera::new();

    let mut opts = ViewOptions { trails: true, orbits: true, galaxy_dust: true, connections: false };

    let mut expansion_speed: f32 = 0.1;
    let mut last_mouse = Vec2::from(mouse_position());

    let mut selected: Option<u64> = None;
    let mut follow_selected = false;
    let mut show_detail_modal = false;
    let mut show_help = true;
    let mut fullscreen = false;

    loop {
        let current_width = screen_width();
        let current_height = screen_height();
        let dt = get_frame_time().min(0.1);

        let mouse = Vec2::from(mouse_position());
        let mouse_delta = mouse - last_mouse;
        last_mouse = mouse;

        if is_key_pressed(KeyCode::Key1) {
            expansion_speed = 0.05;
        }
        if is_key_pressed(KeyCode::Key2) {
            expansion_speed = 0.15;
        }
        if is_key_pressed(KeyCode::Key3) {
            expansion_speed = 0.3;
        }

        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
            next_frame().await;
            continue;
        }
        if is_key_pressed(KeyCode::T) {
            opts.trails = !opts.trails;
        }
        if is_key_pressed(KeyCode::O) {
            opts.orbits = !opts.orbits;
        }
        if is_key_pressed(KeyCode::G) {
            opts.galaxy_dust = !opts.galaxy_dust;
        }
        if is_key_pressed(KeyCode::L) {
            opts.connections = !opts.connections;
        }
        if is_key_pressed(KeyCode::H) {
            show_help = !show_help;
        }
        if is_key_pressed(KeyCode::V) {
            cam.auto_rotate = !cam.auto_rotate;
        }
        if is_key_pressed(KeyCode::C) && selected.is_some() {
            follow_selected = !follow_selected;
        }

        if is_key_pressed(KeyCode::Tab) && selected.is_some() {
            show_detail_modal = true;
        }

        if !show_detail_modal {
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                let focus = cam.ground_point(mouse);
                cam.zoom_by(wheel.signum(), focus);
            }

            if is_mouse_button_down(MouseButton::Right) {
                cam.orbit(mouse_delta.x, mouse_delta.y);
            }
            if is_mouse_button_down(MouseButton::Middle) {
                follow_selected = false;
                cam.pan_screen(mouse_delta.x, mouse_delta.y);
            }

            let mut fwd = 0.0;
            let mut side = 0.0;
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                fwd += 1.0;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                fwd -= 1.0;
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                side += 1.0;
            }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                side -= 1.0;
            }
            if fwd != 0.0 || side != 0.0 {
                follow_selected = false;
                cam.pan_ground(fwd * dt * 0.8, side * dt * 0.8);
            }

            if is_key_pressed(KeyCode::R) {
                follow_selected = false;
                cam.reset();
            }
        }

        let mut time_scale = 1.0;
        if is_key_down(KeyCode::Space) {
            time_scale = 0.0;
        } else if is_key_down(KeyCode::LeftShift) {
            time_scale = 0.3;
        }

        let (ray_o, ray_d) = cam.screen_ray(mouse);
        let over_panel = selected.is_some()
            && Renderer::info_panel_rect(current_width).is_some_and(|r| r.contains(mouse));
        let hovered = if show_detail_modal || over_panel {
            None
        } else {
            universe.pick(ray_o, ray_d, cam.pixel_angle())
        };

        if is_mouse_button_pressed(MouseButton::Left) {
            if show_detail_modal {
                if !rendering::modal_rect(current_width, current_height).contains(mouse) {
                    show_detail_modal = false;
                }
            } else if over_panel {
                if Renderer::info_button_rect(current_width).is_some_and(|b| b.contains(mouse)) {
                    show_detail_modal = true;
                }
            } else {
                selected = hovered;
                if selected.is_none() {
                    follow_selected = false;
                }
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            show_detail_modal = false;
        }

        universe.update(expansion_speed, time_scale);

        if let Some(id) = selected {
            match universe.get_object_by_id(id) {
                Some(obj) => {
                    if follow_selected {
                        cam.set_target(rendering::world_pos(obj));
                    }
                }
                None => {
                    selected = None;
                    follow_selected = false;
                    show_detail_modal = false;
                }
            }
        }

        cam.update(dt);
        renderer.update(&universe, dt * time_scale);

        clear_background(Color::new(0.006, 0.006, 0.018, 1.0));

        renderer.draw_universe(&universe, &cam, &opts, selected, hovered);

        let selected_obj = selected.and_then(|id| universe.get_object_by_id(id));

        if let (Some(obj), false) = (selected_obj, show_detail_modal) {
            let info = rendering::object_info(obj, universe.age);
            renderer.draw_object_info_with_button(&info, current_width);
        }

        if let (Some(obj), true) = (selected_obj, show_detail_modal) {
            renderer.draw_detail_modal(obj, &mut show_detail_modal, current_width, current_height);
        }

        let time_scale_text = match time_scale {
            x if x == 0.0 => "PAUSED",
            x if x < 0.5 => "SLOW",
            _ => "NORMAL",
        };

        if !show_detail_modal && show_help {
            renderer.draw_help(
                expansion_speed,
                universe.age,
                cam.zoom(),
                time_scale_text,
                fullscreen,
                &opts,
                universe.objects.len(),
                current_width,
                current_height,
            );
        }

        if follow_selected && !show_detail_modal {
            let text = "FOLLOWING SELECTED OBJECT (C to stop)";
            let w = measure_text(text, None, 16, 1.0).width;
            draw_text(text, (current_width - w) / 2.0, current_height - 20.0, 16.0, Color::new(1.0, 0.85, 0.4, 0.9));
        }

        next_frame().await;
    }
}
