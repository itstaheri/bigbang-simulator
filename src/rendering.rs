use std::collections::{HashMap, HashSet, VecDeque};
use std::f32::consts::{PI, TAU};

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

use crate::camera::OrbitCamera;
use crate::objects::{CelestialObject, ObjectType};
use crate::universe::Universe;

#[inline]
pub fn world_pos(o: &CelestialObject) -> Vec3 {
    vec3(o.x, o.z, o.y)
}

#[inline]
fn sim_to_world(x: f32, y: f32, z: f32) -> Vec3 {
    vec3(x, z, y)
}

pub fn visual_radius(o: &CelestialObject) -> f32 {
    match o.object_type {
        ObjectType::Planet => o.radius * 2.4,
        ObjectType::Asteroid | ObjectType::Comet => o.radius * 2.0,
        ObjectType::Nebula => o.radius * 2.0,
        ObjectType::DarkMatter => o.radius * 3.0,
        ObjectType::GalaxyCenter => o.radius * 1.5,
        _ => o.radius,
    }
}

#[inline]
fn hash01(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x & 0x00ff_ffff) as f32 / 16_777_216.0
}

#[inline]
fn rgb(c: (u8, u8, u8)) -> Color {
    Color::new(c.0 as f32 / 255.0, c.1 as f32 / 255.0, c.2 as f32 / 255.0, 1.0)
}

#[inline]
fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

#[inline]
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

#[inline]
fn pack(c: Color) -> [u8; 4] {
    [
        (c.r.clamp(0.0, 1.0) * 255.0) as u8,
        (c.g.clamp(0.0, 1.0) * 255.0) as u8,
        (c.b.clamp(0.0, 1.0) * 255.0) as u8,
        (c.a.clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

#[inline]
fn screen_fade(size: f32, dist: f32, pix: f32, start_px: f32, end_px: f32) -> f32 {
    let px = size / (dist * pix).max(1e-6);
    1.0 - smoothstep(start_px, end_px, px)
}

#[inline]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;

varying lowp vec4 color;
varying mediump vec2 uv;
varying mediump vec4 params;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
    params = normal;
}
"#;

const GLOW_FRAGMENT: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying mediump vec2 uv;
varying mediump vec4 params;

void main() {
    vec2 p = uv * 2.0 - 1.0;
    float d2 = dot(p, p);
    float kind = params.w;
    vec3 rgb = color.rgb;
    float a;

    if (kind < 0.5) {
        float core = exp(-d2 * params.x);
        float halo = exp(-d2 * 5.0) * 0.45 + exp(-d2 * 1.6) * 0.10;
        a = core + halo * params.z;
        rgb = mix(rgb, vec3(1.0), clamp(core * params.y, 0.0, 1.0));
        a *= clamp(1.0 - d2, 0.0, 1.0);
    } else if (kind < 1.5) {
        a = exp(-d2 * 3.2) * clamp(1.0 - d2, 0.0, 1.0);
    } else if (kind < 2.5) {
        float d = sqrt(d2);
        float x = (d - params.x) / params.y;
        a = exp(-x * x) * clamp(1.0 - d2, 0.0, 1.0);
    } else if (kind < 3.5) {
        float x = abs(uv.x * 2.0 - 1.0);
        a = (1.0 - x) * (1.0 - x);
    } else {
        a = 1.0;
    }
    gl_FragColor = vec4(rgb, a * color.a);
}
"#;

const SOLID_FRAGMENT: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying mediump vec2 uv;
varying mediump vec4 params;

void main() {
    vec2 p = uv * 2.0 - 1.0;
    float r2 = dot(p, p);
    if (r2 > 1.0) {
        discard;
    }
    vec3 n = vec3(p, sqrt(1.0 - r2));
    float diff = max(dot(n, params.xyz), 0.0);
    float rim = pow(1.0 - n.z, 3.0);
    vec3 base = color.rgb;
    vec3 col = base * (params.w + diff * (1.0 - params.w));
    col += base * rim * (0.15 + 0.85 * diff) * 0.9;
    gl_FragColor = vec4(col, 1.0);
}
"#;

const K_STAR: f32 = 0.0;
const K_PUFF: f32 = 1.0;
const K_RING: f32 = 2.0;
const K_STREAK: f32 = 3.0;
const K_FLAT: f32 = 4.0;

const MAX_BATCH_VERTICES: usize = 3200;
const MAX_BATCH_INDICES: usize = 4800;

struct Batch {
    mesh: Mesh,
    right: Vec3,
    up: Vec3,
    cam_pos: Vec3,
}

impl Batch {
    fn new() -> Self {
        Batch {
            mesh: Mesh {
                vertices: Vec::with_capacity(MAX_BATCH_VERTICES),
                indices: Vec::with_capacity(MAX_BATCH_INDICES),
                texture: None,
            },
            right: Vec3::X,
            up: Vec3::Y,
            cam_pos: Vec3::ZERO,
        }
    }

    fn flush(&mut self) {
        if !self.mesh.indices.is_empty() {
            draw_mesh(&self.mesh);
        }
        self.mesh.vertices.clear();
        self.mesh.indices.clear();
    }

    #[inline]
    fn reserve(&mut self, verts: usize, inds: usize) -> u16 {
        if self.mesh.vertices.len() + verts > MAX_BATCH_VERTICES
            || self.mesh.indices.len() + inds > MAX_BATCH_INDICES
        {
            self.flush();
        }
        self.mesh.vertices.len() as u16
    }

    #[inline]
    fn vertex(&mut self, p: Vec3, uv: Vec2, color: [u8; 4], params: Vec4) {
        self.mesh.vertices.push(Vertex { position: p, uv, color, normal: params });
    }

    fn billboard(&mut self, c: Vec3, size: f32, color: Color, params: Vec4) {
        let base = self.reserve(4, 6);
        let r = self.right * size;
        let u = self.up * size;
        let col = pack(color);
        self.vertex(c - r - u, vec2(0.0, 0.0), col, params);
        self.vertex(c + r - u, vec2(1.0, 0.0), col, params);
        self.vertex(c + r + u, vec2(1.0, 1.0), col, params);
        self.vertex(c - r + u, vec2(0.0, 1.0), col, params);
        self.mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn streak(&mut self, a: Vec3, b: Vec3, wa: f32, wb: f32, ca: Color, cb: Color) {
        let axis = b - a;
        let mid = (a + b) * 0.5;
        let side = axis.cross(self.cam_pos - mid).normalize_or_zero();
        if side == Vec3::ZERO {
            return;
        }
        let base = self.reserve(4, 6);
        let params = vec4(0.0, 0.0, 0.0, K_STREAK);
        let (pa, pb) = (pack(ca), pack(cb));
        self.vertex(a - side * wa, vec2(0.0, 0.0), pa, params);
        self.vertex(a + side * wa, vec2(1.0, 0.0), pa, params);
        self.vertex(b + side * wb, vec2(1.0, 1.0), pb, params);
        self.vertex(b - side * wb, vec2(0.0, 1.0), pb, params);
        self.mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

struct DustParticle {
    radius: f32,
    angle: f32,
    height: f32,
    omega: f32,
    size: f32,
    color: Color,
    cloud: bool,
}

struct SkyStar {
    dir: Vec3,
    px: f32,
    color: Color,
}

struct Trail {
    points: VecDeque<Vec3>,
    color: Color,
}

pub struct ViewOptions {
    pub trails: bool,
    pub orbits: bool,
    pub galaxy_dust: bool,
    pub connections: bool,
}

pub struct Renderer {
    glow: Material,
    solid: Material,
    batch: Batch,
    dust: Vec<DustParticle>,
    sky: Vec<SkyStar>,
    trails: HashMap<u64, Trail>,
    trail_tick: u32,
    pub time: f32,
    host: Vec<Option<usize>>,
    live_ids: HashSet<u64>,
}

const TRAIL_LEN: usize = 64;
const TRAIL_EVERY: u32 = 3;

fn gaussian() -> f32 {
    let u1 = rand::gen_range(1e-6f32, 1.0);
    let u2 = rand::gen_range(0.0f32, 1.0);
    (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
}

impl Renderer {
    pub fn new() -> Self {
        let glow = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: GLOW_FRAGMENT },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::One,
                    )),
                    alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
                    depth_test: Comparison::LessOrEqual,
                    depth_write: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("glow shader failed to compile");

        let solid = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: SOLID_FRAGMENT },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("solid shader failed to compile");

        Renderer {
            glow,
            solid,
            batch: Batch::new(),
            dust: Self::make_galaxy_dust(),
            sky: Self::make_sky(),
            trails: HashMap::new(),
            trail_tick: 0,
            time: 0.0,
            host: Vec::new(),
            live_ids: HashSet::new(),
        }
    }

    fn make_galaxy_dust() -> Vec<DustParticle> {
        let mut out = Vec::with_capacity(5200);
        let inner = Color::new(1.0, 0.82, 0.55, 1.0);
        let outer = Color::new(0.45, 0.6, 1.0, 1.0);
        let pink = Color::new(1.0, 0.4, 0.65, 1.0);

        for i in 0..5200 {
            let cloud = i % 3 != 0;
            let u: f32 = rand::gen_range(0.0, 1.0);
            let radius = 70.0 + 1850.0 * u.powf(1.1);
            let arm = (i % 2) as f32 * PI;
            let spread = if cloud { 0.32 } else { 0.5 };
            let angle = arm + (radius / 60.0).ln() * 2.1 + gaussian() * spread;
            let height = gaussian() * (5.0 + radius * 0.018) * if cloud { 0.7 } else { 1.0 };
            let t = smoothstep(80.0, 1400.0, radius);
            let mut color = lerp_color(inner, outer, t);
            let mut size = if cloud { rand::gen_range(22.0, 60.0) } else { rand::gen_range(1.2, 2.4) };
            if cloud && t > 0.2 && rand::gen_range(0.0f32, 1.0) < 0.14 {
                color = pink;
                size *= 0.6;
            }
            let alpha = if cloud { 0.022 + 0.022 * (1.0 - t) } else { 0.55 };
            out.push(DustParticle {
                radius,
                angle,
                height,
                omega: 0.05 * (120.0 / radius).sqrt(),
                size,
                color: with_alpha(color, alpha),
                cloud,
            });
        }

        for _ in 0..260 {
            let radius = gaussian().abs() * 90.0 + 5.0;
            out.push(DustParticle {
                radius,
                angle: rand::gen_range(0.0, TAU),
                height: gaussian() * radius * 0.45,
                omega: 0.05 * (120.0 / radius.max(30.0)).sqrt(),
                size: rand::gen_range(14.0, 36.0),
                color: Color::new(1.0, 0.8, 0.5, 0.03),
                cloud: true,
            });
        }
        out
    }

    fn make_sky() -> Vec<SkyStar> {
        (0..2600)
            .map(|_| {
                let z: f32 = rand::gen_range(-1.0, 1.0);
                let a: f32 = rand::gen_range(0.0, TAU);
                let r = (1.0 - z * z).sqrt();
                let dir = vec3(r * a.cos(), z, r * a.sin());
                let b: f32 = rand::gen_range(0.0f32, 1.0).powf(3.0);
                let tint = rand::gen_range(0.0f32, 1.0);
                let color = if tint < 0.25 {
                    Color::new(1.0, 0.8, 0.65, 1.0)
                } else if tint < 0.55 {
                    Color::new(0.75, 0.85, 1.0, 1.0)
                } else {
                    Color::new(1.0, 1.0, 1.0, 1.0)
                };
                SkyStar { dir, px: 1.4 + b * 3.0, color: with_alpha(color, 0.25 + b * 0.75) }
            })
            .collect()
    }

    pub fn update(&mut self, universe: &Universe, sim_dt: f32) {
        self.time += sim_dt;
        if sim_dt <= 0.0 {
            return;
        }

        self.trail_tick = self.trail_tick.wrapping_add(1);
        if self.trail_tick % TRAIL_EVERY != 0 {
            return;
        }

        self.live_ids.clear();
        for obj in &universe.objects {
            let color = match obj.object_type {
                ObjectType::Planet => Color::new(0.45, 0.7, 1.0, 1.0),
                ObjectType::Comet => Color::new(0.6, 0.85, 1.0, 1.0),
                ObjectType::Asteroid => Color::new(0.7, 0.7, 0.7, 1.0),
                ObjectType::Star | ObjectType::WhiteDwarf => {
                    let c = rgb(obj.color);
                    Color::new(c.r * 0.9, c.g * 0.8, c.b * 0.9, 1.0)
                }
                ObjectType::NeutronStar | ObjectType::Pulsar => Color::new(0.7, 0.8, 1.0, 1.0),
                _ => continue,
            };
            self.live_ids.insert(obj.id);
            let trail = self.trails.entry(obj.id).or_insert_with(|| Trail {
                points: VecDeque::with_capacity(TRAIL_LEN),
                color,
            });
            if trail.points.len() == TRAIL_LEN {
                trail.points.pop_front();
            }
            trail.points.push_back(world_pos(obj));
        }
        let live = &self.live_ids;
        self.trails.retain(|id, _| live.contains(id));
    }

    pub fn draw_universe(
        &mut self,
        universe: &Universe,
        cam: &OrbitCamera,
        opts: &ViewOptions,
        selected: Option<u64>,
        hovered: Option<u64>,
    ) {
        let cam3d = cam.camera3d();
        set_camera(&cam3d);

        let (_, right, up) = cam.basis();
        let cam_pos = cam3d.position;
        self.batch.right = right;
        self.batch.up = up;
        self.batch.cam_pos = cam_pos;
        let pix = cam.pixel_angle();
        let zoom = cam.zoom();

        self.compute_light_hosts(universe);

        gl_use_material(&self.glow);
        self.draw_sky(cam_pos, pix);
        self.batch.flush();

        gl_use_material(&self.solid);
        self.draw_solids(universe, cam_pos, right, up, pix);
        self.batch.flush();

        gl_use_default_material();
        if opts.trails {
            self.draw_trails();
        }
        if opts.orbits && zoom > 1.5 {
            self.draw_orbits(universe, zoom);
        }
        if opts.connections && zoom > 2.5 {
            draw_object_connections(&universe.objects);
        }

        gl_use_material(&self.glow);
        if opts.galaxy_dust {
            self.draw_galaxy_dust(cam_pos, pix);
        }
        for obj in &universe.objects {
            self.draw_glow_object(obj, universe, cam_pos, pix);
        }
        if let Some(id) = hovered.filter(|h| Some(*h) != selected) {
            if let Some(obj) = universe.get_object_by_id(id) {
                self.draw_marker(obj, cam_pos, pix, Color::new(0.6, 0.85, 1.0, 0.45), 1.0);
            }
        }
        if let Some(id) = selected {
            if let Some(obj) = universe.get_object_by_id(id) {
                let pulse = 0.75 + 0.25 * (get_time() as f32 * 4.0).sin();
                self.draw_marker(obj, cam_pos, pix, Color::new(1.0, 0.85, 0.3, 0.9 * pulse), 1.0 + 0.08 * pulse);
            }
        }
        self.batch.flush();

        gl_use_default_material();
        set_default_camera();
    }

    fn compute_light_hosts(&mut self, universe: &Universe) {
        let objs = &universe.objects;
        self.host.clear();
        self.host.resize(objs.len(), None);

        for (i, o) in objs.iter().enumerate() {
            if !matches!(o.object_type, ObjectType::Planet | ObjectType::Asteroid | ObjectType::Comet) {
                continue;
            }
            let mut best = (0.0f32, None);
            for (j, s) in objs.iter().enumerate() {
                if !matches!(
                    s.object_type,
                    ObjectType::Star | ObjectType::WhiteDwarf | ObjectType::GalaxyCenter | ObjectType::NeutronStar | ObjectType::Pulsar
                ) {
                    continue;
                }
                let d2 = (s.x - o.x).powi(2) + (s.y - o.y).powi(2) + (s.z - o.z).powi(2) + 1.0;
                let score = s.mass / d2;
                if score > best.0 {
                    best = (score, Some(j));
                }
            }
            self.host[i] = best.1;
        }
    }

    fn draw_sky(&mut self, cam_pos: Vec3, pix: f32) {
        const R: f32 = 90_000.0;
        let params = vec4(40.0, 1.0, 0.4, K_STAR);
        for i in 0..self.sky.len() {
            let s = &self.sky[i];
            let (dir, px, color) = (s.dir, s.px, s.color);
            self.batch.billboard(cam_pos + dir * R, R * pix * px, color, params);
        }
        for i in 0..14u32 {
            let z = hash01(i * 7 + 1) * 2.0 - 1.0;
            let a = hash01(i * 13 + 5) * TAU;
            let r = (1.0 - z * z).sqrt();
            let dir = vec3(r * a.cos(), z, r * a.sin());
            let hue = hash01(i * 31 + 9);
            let color = if hue < 0.33 {
                Color::new(0.6, 0.4, 1.0, 0.10)
            } else if hue < 0.66 {
                Color::new(1.0, 0.5, 0.6, 0.08)
            } else {
                Color::new(0.4, 0.7, 1.0, 0.09)
            };
            let size = R * (0.03 + hash01(i * 17 + 3) * 0.06);
            self.batch.billboard(cam_pos + dir * R, size, color, vec4(0.0, 0.0, 0.0, K_PUFF));
        }
    }

    fn draw_galaxy_dust(&mut self, cam_pos: Vec3, pix: f32) {
        let t = self.time;
        for i in 0..self.dust.len() {
            let d = &self.dust[i];
            let a = d.angle + d.omega * t;
            let p = vec3(d.radius * a.cos(), d.height, d.radius * a.sin());
            let dist = (p - cam_pos).length();
            if d.cloud {
                let fade = smoothstep(d.size * 2.0, d.size * 8.0, dist) * screen_fade(d.size, dist, pix, 90.0, 350.0);
                if fade <= 0.01 {
                    continue;
                }
                let c = with_alpha(d.color, d.color.a * fade);
                self.batch.billboard(p, d.size, c, vec4(0.0, 0.0, 0.0, K_PUFF));
            } else {
                let size = (dist * pix * 2.2).max(d.size);
                let tw = 0.8 + 0.2 * (t * 2.0 + i as f32).sin();
                let c = with_alpha(d.color, d.color.a * tw);
                self.batch.billboard(p, size, c, vec4(60.0, 1.0, 0.25, K_STAR));
            }
        }
    }

    fn draw_solids(&mut self, universe: &Universe, cam_pos: Vec3, right: Vec3, up: Vec3, pix: f32) {
        for (i, obj) in universe.objects.iter().enumerate() {
            let (base, ambient) = match obj.object_type {
                ObjectType::Planet => (rgb(obj.color), 0.06),
                ObjectType::Asteroid => (Color::new(0.55, 0.52, 0.5, 1.0), 0.08),
                ObjectType::Comet => (Color::new(0.75, 0.85, 0.95, 1.0), 0.15),
                ObjectType::BlackHole => (BLACK, 0.0),
                _ => continue,
            };
            let p = world_pos(obj);
            let to_cam = cam_pos - p;
            let dist = to_cam.length();
            let min_px = if obj.object_type == ObjectType::Asteroid { 1.3 } else { 2.0 };
            let size = visual_radius(obj).max(dist * pix * min_px);

            let light = match self.host.get(i).copied().flatten() {
                Some(h) => {
                    let l = (world_pos(&universe.objects[h]) - p).normalize_or_zero();
                    let fwd = to_cam / dist.max(1e-4);
                    vec3(l.dot(right), l.dot(up), l.dot(fwd))
                }
                None => Vec3::ZERO,
            };
            self.batch.billboard(p, size, base, vec4(light.x, light.y, light.z, ambient));
        }
    }

    fn draw_trails(&self) {
        for trail in self.trails.values() {
            let n = trail.points.len();
            if n < 2 {
                continue;
            }
            for k in 1..n {
                let a = k as f32 / n as f32;
                let c = with_alpha(trail.color, a * a * 0.45);
                draw_line_3d(trail.points[k - 1], trail.points[k], c);
            }
        }
    }

    fn draw_orbits(&self, universe: &Universe, zoom: f32) {
        let objs = &universe.objects;
        let base_alpha = if zoom > 3.0 { 0.35 } else if zoom > 2.0 { 0.28 } else { 0.2 };

        for (i, planet) in objs.iter().enumerate() {
            if planet.object_type != ObjectType::Planet {
                continue;
            }
            let Some(h) = self.host.get(i).copied().flatten() else { continue };
            let star = &objs[h];
            if star.object_type == ObjectType::GalaxyCenter {
                continue;
            }

            let r = vec3(planet.x - star.x, planet.y - star.y, planet.z - star.z);
            let distance = r.length();
            if !(8.0..80.0).contains(&distance) {
                continue;
            }
            let v = vec3(planet.vx - star.vx, planet.vy - star.vy, planet.vz - star.vz);
            let normal = r.cross(v).normalize_or_zero();
            let normal = if normal == Vec3::ZERO { Vec3::Z } else { normal };
            let u = r / distance;
            let w = normal.cross(u);

            let color = if distance < 30.0 {
                Color::new(0.3, 0.6, 1.0, base_alpha * 1.5)
            } else if distance < 50.0 {
                Color::new(0.2, 0.5, 0.9, base_alpha * 1.2)
            } else {
                Color::new(0.1, 0.4, 0.8, base_alpha)
            };

            let center = world_pos(star);
            let segments = 72;
            let mut prev = center + sim_vec(u * distance);
            for s in 1..=segments {
                let a = s as f32 / segments as f32 * TAU;
                let p = center + sim_vec((u * a.cos() + w * a.sin()) * distance);
                draw_line_3d(prev, p, color);
                prev = p;
            }
        }
    }

    fn draw_marker(&mut self, obj: &CelestialObject, cam_pos: Vec3, pix: f32, color: Color, scale: f32) {
        let p = world_pos(obj);
        let dist = (p - cam_pos).length();
        let body = match obj.object_type {
            ObjectType::Nebula | ObjectType::GalaxyCenter | ObjectType::DarkMatter => visual_radius(obj) * 1.2,
            _ => visual_radius(obj) * 2.2,
        };
        let size = body.max(dist * pix * 16.0) * scale;
        self.batch.billboard(p, size, color, vec4(0.78, 0.035, 0.0, K_RING));
    }

    fn draw_glow_object(&mut self, obj: &CelestialObject, universe: &Universe, cam_pos: Vec3, pix: f32) {
        let p = world_pos(obj);
        let to_cam = cam_pos - p;
        let dist = to_cam.length().max(1e-3);
        let t = self.time;
        let seed = obj.visual_seed;

        match obj.object_type {
            ObjectType::Star => {
                let c = rgb(obj.color);
                let lum = obj.luminosity.max(0.0);
                let boost = (1.0 + lum).ln();
                let r = obj.radius * (1.0 + 0.18 * boost);
                let natural = r * 9.0;
                let min = dist * pix * 11.0;
                let size = natural.max(min).min(dist * pix * 140.0);
                let halo = (natural / size).sqrt().clamp(0.35, 1.0);
                let tw = 0.92 + 0.08 * (t * 3.0 + seed).sin();
                let a = ((0.55 + 0.14 * boost) * tw).min(1.0);
                self.batch.billboard(p, size, with_alpha(c, a), vec4(60.0, 0.9, halo, K_STAR));

                if lum > 5.0 {
                    let corona = size * 3.5;
                    let fade = screen_fade(corona, dist, pix, 150.0, 450.0);
                    if fade > 0.01 {
                        self.batch.billboard(p, corona, with_alpha(c, 0.10 * fade), vec4(0.0, 0.0, 0.0, K_PUFF));
                    }
                    let len = (size * 2.2).min(dist * pix * 220.0);
                    let w = (size * 0.035).min(dist * pix * 2.0);
                    let (rv, uv) = (self.batch.right, self.batch.up);
                    let spike = with_alpha(lerp_color(c, WHITE, 0.5), 0.45);
                    let clear = with_alpha(c, 0.0);
                    for axis in [rv, uv] {
                        self.batch.streak(p, p + axis * len, w, 0.0, spike, clear);
                        self.batch.streak(p, p - axis * len, w, 0.0, spike, clear);
                    }
                }
            }
            ObjectType::WhiteDwarf => {
                let size = (obj.radius * 7.0).max(dist * pix * 9.0);
                self.batch.billboard(p, size, Color::new(0.85, 0.9, 1.0, 0.9), vec4(90.0, 1.0, 0.8, K_STAR));
            }
            ObjectType::NeutronStar => {
                let size = (obj.radius * 12.0).max(dist * pix * 9.0);
                self.batch.billboard(p, size, Color::new(0.75, 0.82, 1.0, 1.0), vec4(120.0, 1.0, 0.9, K_STAR));
                let pulse = 0.5 + 0.5 * (t * 6.0 + seed).sin();
                self.batch.billboard(
                    p,
                    size * 0.9,
                    Color::new(0.5, 0.65, 1.0, 0.25 + 0.25 * pulse),
                    vec4(0.55 + 0.1 * pulse, 0.05, 0.0, K_RING),
                );
            }
            ObjectType::Pulsar => {
                let size = (obj.radius * 12.0).max(dist * pix * 9.0);
                self.batch.billboard(p, size, Color::new(0.7, 0.85, 1.0, 1.0), vec4(120.0, 1.0, 0.9, K_STAR));

                let spin = vec3(hash01(seed as u32) - 0.5, 1.0, hash01(seed as u32 + 7) - 0.5).normalize();
                let (bu, bw) = basis_of(spin);
                let omega = (1.0 / obj.rotation_period.max(1e-4)).ln().max(1.0) * 1.5;
                let ang = t * omega + seed;
                let tilt = 35.0_f32.to_radians();
                let beam = spin * tilt.cos() + (bu * ang.cos() + bw * ang.sin()) * tilt.sin();
                let len = 70.0f32.max(dist * pix * 60.0);
                let w0 = (0.5f32).max(dist * pix * 1.5);
                let hot = Color::new(0.7, 0.85, 1.0, 0.85);
                let clear = Color::new(0.3, 0.5, 1.0, 0.0);
                self.batch.streak(p, p + beam * len, w0, w0 * 6.0, hot, clear);
                self.batch.streak(p, p - beam * len, w0, w0 * 6.0, hot, clear);
            }
            ObjectType::GalaxyCenter => {
                let back = p - to_cam / dist * (obj.radius * 2.0);
                let warm = Color::new(1.0, 0.88, 0.62, 1.0);
                let layers = [
                    (obj.radius * 6.0, with_alpha(warm, 0.75), vec4(6.0, 0.6, 0.8, K_STAR)),
                    (obj.radius * 20.0, with_alpha(warm, 0.12), vec4(0.0, 0.0, 0.0, K_PUFF)),
                    (obj.radius * 55.0, Color::new(0.9, 0.6, 0.45, 0.035), vec4(0.0, 0.0, 0.0, K_PUFF)),
                ];
                for (size, color, params) in layers {
                    let fade = screen_fade(size, dist, pix, 160.0, 700.0).max(0.12);
                    self.batch.billboard(back, size, with_alpha(color, color.a * fade), params);
                }
            }
            ObjectType::BlackHole => {
                self.draw_accretion_disk(obj, cam_pos, pix);
                let r = visual_radius(obj).max(dist * pix * 2.0);
                self.batch.billboard(p, r * 1.6, Color::new(1.0, 0.75, 0.45, 0.9), vec4(0.7, 0.07, 0.0, K_RING));
            }
            ObjectType::Planet => {
                let r = visual_radius(obj).max(dist * pix * 2.0);
                let c = rgb(obj.color);
                self.batch.billboard(p, r * 1.5, with_alpha(lerp_color(c, WHITE, 0.4), 0.22), vec4(0.68, 0.09, 0.0, K_RING));
            }
            ObjectType::Comet => self.draw_comet(obj, universe, cam_pos, pix),
            ObjectType::Asteroid => {}
            ObjectType::DarkMatter => {
                let r = visual_radius(obj);
                self.batch.billboard(p, r * 2.0, Color::new(0.35, 0.35, 0.8, 0.30), vec4(0.7, 0.10, 0.0, K_RING));
                self.batch.billboard(p, r * 5.0, Color::new(0.3, 0.3, 0.7, 0.07), vec4(0.0, 0.0, 0.0, K_PUFF));
            }
            ObjectType::DarkEnergy => {
                let size = (obj.radius * 0.5).max(dist * pix * 3.0);
                self.batch.billboard(p, size, Color::new(0.6, 0.2, 0.7, 0.4), vec4(40.0, 0.2, 0.5, K_STAR));
            }
            ObjectType::Nebula => self.draw_nebula(obj, cam_pos, pix),
        }
    }

    fn draw_accretion_disk(&mut self, obj: &CelestialObject, cam_pos: Vec3, pix: f32) {
        let p = world_pos(obj);
        let dist = (cam_pos - p).length();
        let r0 = visual_radius(obj).max(dist * pix * 2.0);
        let radii = [r0 * 1.5, r0 * 2.6, r0 * 4.5, r0 * 7.5];
        let to_cam = (cam_pos - p).normalize_or_zero();
        let t = self.time;
        const SEG: usize = 64;

        let hot = Color::new(1.0, 0.92, 0.75, 1.0);
        let cool = Color::new(0.95, 0.35, 0.08, 1.0);

        let ring_count = radii.len();
        let base = self.batch.reserve(SEG * ring_count, (SEG) * (ring_count - 1) * 6);
        let params = vec4(0.0, 0.0, 0.0, K_FLAT);
        for s in 0..SEG {
            let a = s as f32 / SEG as f32 * TAU;
            let (sa, ca) = a.sin_cos();
            let tangent = vec3(-sa, 0.0, ca);
            let beaming = 1.0 + 0.7 * tangent.dot(to_cam);
            for (k, &r) in radii.iter().enumerate() {
                let f = k as f32 / (ring_count - 1) as f32;
                let streaks = 0.75 + 0.25 * (a * 7.0 - t * 2.5 + f * 4.0).sin();
                let alpha = match k {
                    0 => 0.0,
                    _ => (1.0 - f).powf(1.2) * 0.95 * beaming * streaks + if k == 1 { 0.1 } else { 0.0 },
                };
                let c = with_alpha(lerp_color(hot, cool, f), alpha.min(1.0));
                self.batch.vertex(p + vec3(ca * r, 0.0, sa * r), vec2(0.0, 0.0), pack(c), params);
            }
        }
        for s in 0..SEG {
            let s2 = (s + 1) % SEG;
            for k in 0..ring_count - 1 {
                let i0 = base + (s * ring_count + k) as u16;
                let i1 = base + (s * ring_count + k + 1) as u16;
                let j0 = base + (s2 * ring_count + k) as u16;
                let j1 = base + (s2 * ring_count + k + 1) as u16;
                self.batch.mesh.indices.extend_from_slice(&[i0, j0, j1, i0, j1, i1]);
            }
        }
    }

    fn draw_comet(&mut self, obj: &CelestialObject, universe: &Universe, cam_pos: Vec3, pix: f32) {
        let p = world_pos(obj);
        let dist = (cam_pos - p).length();
        let idx = universe.objects.iter().position(|o| o.id == obj.id);
        let sun = idx
            .and_then(|i| self.host.get(i).copied().flatten())
            .map(|h| world_pos(&universe.objects[h]))
            .unwrap_or(Vec3::ZERO);
        let away = (p - sun).normalize_or_zero();
        let vel = sim_to_world(obj.vx, obj.vy, obj.vz).normalize_or_zero();

        let coma = (visual_radius(obj) * 5.0).max(dist * pix * 7.0);
        self.batch.billboard(p, coma, Color::new(0.6, 0.85, 1.0, 0.75), vec4(30.0, 0.8, 0.9, K_STAR));

        let len = 45.0f32.max(dist * pix * 40.0);
        let w = coma * 0.35;
        self.batch.streak(p, p + away * len, w * 0.5, w * 1.2, Color::new(0.4, 0.7, 1.0, 0.6), Color::new(0.3, 0.5, 1.0, 0.0));
        let dust_dir = (away * 0.6 - vel * 0.4).normalize_or_zero();
        self.batch.streak(p, p + dust_dir * len * 0.7, w * 0.6, w * 2.2, Color::new(1.0, 0.9, 0.75, 0.35), Color::new(1.0, 0.85, 0.6, 0.0));
    }

    fn draw_nebula(&mut self, obj: &CelestialObject, cam_pos: Vec3, pix: f32) {
        let center = world_pos(obj);
        let base = rgb(obj.color);
        let radius = obj.radius;
        let density = obj.nebula_density.max(0.1);

        let core_dist = (center - cam_pos).length();
        let core_fade = smoothstep(radius * 0.5, radius * 3.0, core_dist);
        self.batch.billboard(center, radius * 2.6, with_alpha(base, 0.16 * core_fade), vec4(0.0, 0.0, 0.0, K_PUFF));

        let limit = radius * 2.2;
        for part in &obj.nebula_particles {
            let local = vec3(part.x, part.y, part.z);
            let edge = 1.0 - (local.length() / limit).min(1.0).powi(2);
            if edge <= 0.0 {
                continue;
            }
            let wp = center + sim_to_world(part.x, part.y, part.z);
            let size = radius * (0.42 + 0.28 * part.size / density);
            let dist = (wp - cam_pos).length();
            let fade = smoothstep(size * 0.6, size * 3.0, dist) * screen_fade(size, dist, pix, 200.0, 700.0);
            if fade <= 0.01 {
                continue;
            }
            let r = base.r * (0.7 + (part.x.sin() * 0.3).abs());
            let g = base.g * (0.6 + (part.y.cos() * 0.4).abs());
            let b = base.b * (0.8 + ((part.x + part.y) * 0.2).sin().abs());
            let a = 0.15 * density.min(1.3) * edge * fade;
            self.batch.billboard(wp, size, Color::new(r, g, b, a), vec4(0.0, 0.0, 0.0, K_PUFF));
        }

        let seed = obj.visual_seed as u32;
        for k in 0..5u32 {
            let h = |n: u32| hash01(seed.wrapping_mul(31).wrapping_add(k * 97 + n)) * 2.0 - 1.0;
            let off = vec3(h(1), h(2) * 0.4, h(3)) * radius * 0.8;
            let sp = center + off;
            let tw = 0.6 + 0.4 * (self.time * (1.5 + k as f32 * 0.4) + k as f32 * 2.1).sin();
            let size = (radius * 0.12).max((sp - cam_pos).length() * pix * 8.0);
            self.batch.billboard(sp, size, Color::new(1.0, 0.92, 0.85, 0.7 * tw), vec4(70.0, 1.0, 0.6, K_STAR));
        }
    }

    pub fn info_panel_rect(screen_width: f32) -> Option<Rect> {
        if screen_width < 400.0 {
            return None;
        }
        Some(Rect::new(20.0, 20.0, 350.0_f32.min(screen_width * 0.3), 300.0))
    }

    pub fn info_button_rect(screen_width: f32) -> Option<Rect> {
        Self::info_panel_rect(screen_width).map(|r| Rect::new(r.x + 10.0, r.y + r.h - 40.0, r.w - 20.0, 30.0))
    }

    pub fn draw_object_info_with_button(&self, info: &str, screen_width: f32) {
        let Some(r) = Self::info_panel_rect(screen_width) else { return };

        draw_panel(r.x, r.y, r.w, r.h);
        draw_text("OBJECT INFORMATION", r.x + 12.0, r.y + 30.0, 20.0, Color::new(1.0, 0.9, 0.55, 1.0));
        draw_line(r.x + 12.0, r.y + 40.0, r.x + r.w - 12.0, r.y + 40.0, 1.0, Color::new(0.5, 0.7, 1.0, 0.35));

        let mut current_y = r.y + 62.0;
        for line in info.split('\n') {
            let text_height = draw_text_wrapped(line, r.x + 12.0, current_y, 16.0, WHITE, r.w - 24.0);
            current_y += text_height + 3.0;
            if current_y > r.y + r.h - 50.0 {
                break;
            }
        }

        let b = Self::info_button_rect(screen_width).unwrap();
        let hovered = b.contains(mouse_position().into());
        let bg = if hovered { Color::new(0.15, 0.6, 0.3, 0.95) } else { Color::new(0.1, 0.45, 0.22, 0.9) };
        draw_rectangle(b.x, b.y, b.w, b.h, bg);
        draw_rectangle_lines(b.x, b.y, b.w, b.h, 1.5, Color::new(0.3, 0.85, 0.4, 1.0));
        let label = "LEARN MORE (PRESS Tab)";
        let tw = measure_text_width(label, 16.0);
        draw_text(label, b.x + (b.w - tw) / 2.0, b.y + 20.0, 16.0, Color::new(1.0, 1.0, 0.85, 1.0));
    }

    pub fn draw_detail_modal(&self, obj: &CelestialObject, show_modal: &mut bool, screen_width: f32, screen_height: f32) {
        draw_rectangle(0.0, 0.0, screen_width, screen_height, Color::new(0.0, 0.0, 0.0, 0.7));

        let m = modal_rect(screen_width, screen_height);
        draw_rectangle(m.x, m.y, m.w, m.h, Color::new(0.04, 0.05, 0.1, 0.96));
        for i in 1..=3 {
            let g = i as f32 * 4.0;
            draw_rectangle_lines(m.x - g, m.y - g, m.w + 2.0 * g, m.h + 2.0 * g, 2.0, Color::new(0.3, 0.6, 1.0, 0.12 / i as f32));
        }
        draw_rectangle_lines(m.x, m.y, m.w, m.h, 2.0, Color::new(0.6, 0.8, 1.0, 0.9));

        let title = format!("{} - Detailed Information", obj.object_type.name().to_uppercase());
        let title_font_size = if m.w < 500.0 { 22.0 } else { 26.0 };
        let title_width = measure_text_width(&title, title_font_size);
        draw_text(&title, m.x + (m.w - title_width) / 2.0, m.y + 40.0, title_font_size, Color::new(1.0, 1.0, 0.6, 1.0));

        let description = get_detailed_description(obj);
        let font_size = if m.w < 500.0 { 16.0 } else { 18.0 };
        let mut y = m.y + 80.0;
        'outer: for paragraph in description.split("\n\n") {
            let mut chunks: Vec<String> = Vec::new();
            for line in paragraph.lines() {
                let line = line.trim();
                match chunks.last_mut() {
                    Some(last) if !line.starts_with("- ") => {
                        last.push(' ');
                        last.push_str(line);
                    }
                    _ => chunks.push(line.to_string()),
                }
            }
            for chunk in chunks {
                y += draw_text_wrapped(&chunk, m.x + 20.0, y, font_size, WHITE, m.w - 40.0);
                if y > m.y + m.h - 70.0 {
                    break 'outer;
                }
            }
            y += font_size * 0.5;
        }

        let b = close_button_rect(m);
        let hovered = b.contains(mouse_position().into());
        let bg = if hovered { Color::new(0.8, 0.3, 0.3, 0.95) } else { Color::new(0.6, 0.2, 0.2, 0.9) };
        draw_rectangle(b.x, b.y, b.w, b.h, bg);
        draw_rectangle_lines(b.x, b.y, b.w, b.h, 1.5, Color::new(1.0, 0.4, 0.4, 1.0));
        let close_text = "CLOSE";
        let cw = measure_text_width(close_text, 18.0);
        draw_text(close_text, b.x + (b.w - cw) / 2.0, b.y + 23.0, 18.0, Color::new(1.0, 1.0, 0.8, 1.0));

        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            *show_modal = false;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_help(
        &self,
        expansion_speed: f32,
        universe_age: f64,
        zoom: f32,
        time_scale: &str,
        fullscreen: bool,
        opts: &ViewOptions,
        object_count: usize,
        screen_width: f32,
        screen_height: f32,
    ) {
        if screen_width < 600.0 {
            self.draw_minimal_help(expansion_speed, universe_age, zoom, time_scale, screen_width);
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
        let on = |b: bool| if b { "ON" } else { "OFF" };

        let lines: Vec<(String, Color)> = vec![
            ("CONTROLS".into(), Color::new(1.0, 1.0, 0.6, 1.0)),
            (format!("Expansion: {}", speed_text), Color::new(0.6, 1.0, 0.6, 1.0)),
            ("1: Minimal  2: Slow  3: Medium".into(), WHITE),
            (format!("Time: {} (Space: Pause)", time_scale), Color::new(0.6, 1.0, 0.6, 1.0)),
            ("Shift: Slow Motion".into(), WHITE),
            ("Rotate: Right Drag   Pan: Middle Drag".into(), WHITE),
            ("Zoom: Wheel   Move: W A S D / Arrows".into(), WHITE),
            ("Select: Left Click   Details: Tab".into(), Color::new(0.8, 0.9, 1.0, 1.0)),
            ("Follow selected: C   Reset view: R".into(), Color::new(0.8, 0.9, 1.0, 1.0)),
            (format!("Trails T:{}  Orbits O:{}  Dust G:{}", on(opts.trails), on(opts.orbits), on(opts.galaxy_dust)),
                Color::new(0.8, 0.9, 1.0, 1.0)),
            (format!("Links L:{}  Auto-rotate: V", on(opts.connections)), Color::new(0.8, 0.9, 1.0, 1.0)),
            (format!("Fullscreen: F (Now: {})   Hide: H", on(fullscreen)), Color::new(0.8, 0.9, 1.0, 1.0)),
            (format!("Age: {}", age_display), WHITE),
            (format!("Zoom: {:.1}x   Objects: {}   FPS: {}", zoom, object_count, get_fps()), WHITE),
        ];

        let line_h = 21.0;
        let w = 330.0;
        let h = lines.len() as f32 * line_h + 16.0;
        let x = screen_width - w - 16.0;
        let mut y = 16.0;
        if screen_height < 600.0 {
            y = (screen_height - h - 10.0).max(0.0);
        }
        draw_panel(x, y, w, h);
        for (i, (text, color)) in lines.iter().enumerate() {
            let size = if i == 0 { 20.0 } else { 16.0 };
            draw_text(text, x + 12.0, y + 24.0 + i as f32 * line_h, size, *color);
        }
    }

    fn draw_minimal_help(&self, expansion_speed: f32, universe_age: f64, zoom: f32, time_scale: &str, screen_width: f32) {
        let lines = [
            "CONTROLS (Minimal)".to_string(),
            format!(
                "Exp: {} | Time: {}",
                match expansion_speed {
                    x if x < 0.1 => "Min",
                    x if x < 0.2 => "Slow",
                    _ => "Med",
                },
                time_scale
            ),
            "Zoom: Wheel | Rotate: Right".to_string(),
            "Select: Left | Reset: R".to_string(),
            "Details: Tab | Hide: H".to_string(),
            "Fullscreen: F".to_string(),
            format!("Age: {:.1}M | Zoom: {:.1}x", universe_age, zoom),
        ];
        let x = screen_width - 200.0;
        draw_panel(x - 6.0, 0.0, 206.0, lines.len() as f32 * 18.0 + 10.0);
        for (i, text) in lines.iter().enumerate() {
            let font_size = if i == 0 { 14.0 } else { 12.0 };
            let color = if i == 0 { Color::new(1.0, 1.0, 0.6, 1.0) } else { Color::new(0.8, 0.9, 1.0, 1.0) };
            draw_text(text, x, 14.0 + i as f32 * 18.0, font_size, color);
        }
    }
}

#[inline]
fn sim_vec(v: Vec3) -> Vec3 {
    vec3(v.x, v.z, v.y)
}

fn basis_of(n: Vec3) -> (Vec3, Vec3) {
    let helper = if n.x.abs() < 0.9 { Vec3::X } else { Vec3::Z };
    let u = n.cross(helper).normalize();
    (u, n.cross(u))
}

fn draw_object_connections(objects: &[CelestialObject]) {
    for i in 0..objects.len() {
        let a = &objects[i];
        for b in &objects[i + 1..] {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let dz = b.z - a.z;
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 < 30.0 * 30.0 && d2 > 5.0 * 5.0 {
                let alpha = (0.2 / d2.sqrt()).min(0.1) * 2.0;
                draw_line_3d(world_pos(a), world_pos(b), Color::new(0.3, 0.5, 0.8, alpha));
            }
        }
    }
}

pub fn modal_rect(screen_width: f32, screen_height: f32) -> Rect {
    let w = 700.0_f32.min(screen_width * 0.85).max(300.0);
    let h = 520.0_f32.min(screen_height * 0.85).max(300.0);
    Rect::new((screen_width - w) / 2.0, (screen_height - h) / 2.0, w, h)
}

fn close_button_rect(m: Rect) -> Rect {
    let w = 100.0_f32.min(m.w * 0.3);
    Rect::new(m.x + (m.w - w) / 2.0, m.y + m.h - 50.0, w, 35.0)
}

fn draw_panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::new(0.02, 0.03, 0.08, 0.78));
    draw_rectangle_lines(x, y, w, h, 1.5, Color::new(0.45, 0.65, 1.0, 0.55));
}

fn measure_text_width(text: &str, font_size: f32) -> f32 {
    measure_text(text, None, font_size as u16, 1.0).width
}

fn draw_text_wrapped(text: &str, x: f32, y: f32, font_size: f32, color: Color, max_width: f32) -> f32 {
    let mut current_line = String::new();
    let mut current_y = y;
    let line_height = font_size * 1.2;

    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() { word.to_string() } else { format!("{} {}", current_line, word) };
        if measure_text_width(&test_line, font_size) <= max_width {
            current_line = test_line;
        } else {
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
    current_y - y
}

pub fn object_info(obj: &CelestialObject, universe_age: f64) -> String {
    let extra_info = match obj.object_type {
        ObjectType::NeutronStar => format!("\nRotation: {:.3}s\nMag Field: {:.0e} T", obj.rotation_period, obj.magnetic_field),
        ObjectType::Pulsar => format!(
            "\nRotation: {:.3}s\nMag Field: {:.0e} T\nBeam Period: {:.2}s",
            obj.rotation_period, obj.magnetic_field, 1.0 / obj.rotation_period
        ),
        ObjectType::WhiteDwarf => "\nType: Degenerate Star\nDensity: ~1 ton/cm³".to_string(),
        ObjectType::BlackHole => format!("\nEvent Horizon: {:.2} km", obj.event_horizon_radius / 1000.0),
        _ => String::new(),
    };

    format!(
        "Type: {}\nMass: {:.2e} kg\nRadius: {:.1} km\nSpeed: {:.2} km/s\nTemperature: {:.0} K\nAge: {:.1} Myr\nPosition: ({:.0}, {:.0}, {:.0}) km{}",
        obj.object_type.name(),
        obj.mass,
        obj.radius / 1000.0,
        obj.speed() / 1000.0,
        obj.temperature,
        obj.get_age(universe_age) / (1e6 * 365.0 * 24.0 * 3600.0),
        obj.x / 1000.0,
        obj.y / 1000.0,
        obj.z / 1000.0,
        extra_info
    )
}

fn get_detailed_description(obj: &CelestialObject) -> String {
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
            obj.mass / 2e5,
            obj.temperature
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
            obj.speed() / 1000.0,
            obj.temperature
        ),

        ObjectType::Asteroid => format!(
            "ASTEROIDS are minor planets of the inner Solar System, ranging in size\n\
            from about 1 meter to hundreds of kilometers.\n\n\
            This asteroid has a mass of {:.1} tons and is traveling at {:.1} km/s.\n\n\
            Most asteroids are found in the asteroid belt between Mars and Jupiter.\n\
            They are remnants from the Solar System's formation that never coalesced into planets.",
            obj.mass,
            obj.speed() / 1000.0
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

        ObjectType::Nebula => format!(
            "NEBULAE are vast clouds of dust and gas in interstellar space, often\n\
            serving as stellar nurseries where new stars are born.\n\n\
            This nebula has a diameter of {:.0} light-years and a density of {:.2}.\n\
            Its temperature is {:.0} K.\n\n\
            Nebulae come in various types:\n\
            - Emission nebulae: Glow from ionized gases\n\
            - Reflection nebulae: Reflect light from nearby stars\n\
            - Dark nebulae: Dense clouds that block light\n\
            - Planetary nebulae: Formed from dying stars\n\n\
            The beautiful colors of nebulae come from different elements:\n\
            Hydrogen (red), Oxygen (green/blue), and Sulfur (red).\n\n\
            Within this nebula, new stars are being born from collapsing\n\
            regions of gas and dust, continuing the cosmic cycle of\n\
            stellar life and death.",
            obj.radius * 2.0,
            obj.nebula_density,
            obj.temperature
        ),
    }
}
