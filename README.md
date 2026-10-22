### Big Bang Simulator 3D

**Note:** Welcome to my personal cosmic simulation! This is an interactive universe model that I designed and built for exploration, observation, and—above all—fun :)

> **New in v0.2:** The whole simulator is now **3D**: a flared galactic disk, solar systems on tilted orbital planes, volumetric nebulae, a cinematic orbit camera, GPU shaders, and fixes for the bugs found in the 2D version. See [What's New](#-whats-new-in-v02-3d).

#### **What You Can Experience**
**A Stable, Interactive 3D Universe**
- Stars, planets, black holes, neutron stars, pulsars, white dwarfs, comets, asteroids, nebulae and dark matter
- A spiral galaxy whose disk gets thicker towards the edge, with a central black hole
- Solar systems, each with its own tilted orbital plane
- Functional orbital mechanics in three dimensions
- Fly freely around the scene: rotate, zoom towards the cursor, pan, follow an object, or let the camera auto-rotate
- Select any object in 3D to view its properties and parameters

**Note:** This simulation is designed for stability and observation. Stars and planets do not undergo catastrophic events. Objects interact through moderated gravitational forces, and close encounters are resolved with gentle elastic collisions.

#### **🆕 What's New in v0.2 (3D)**
**3D world**
- All physics (gravity, velocities, collisions, expansion) now runs in 3D
- Thin, flared galactic disk; solar systems on randomly tilted orbital planes; dark matter in a puffy halo
- Nebulae are 3D clouds of drifting gas puffs

**Camera & interaction**
- Smooth turntable camera with frame-rate-independent easing
- Zoom towards the point under the mouse
- 3D ray picking, so even tiny or distant objects stay clickable; solid bodies win over the gas clouds around them
- Follow mode (C) keeps the selected object centred
- Auto-rotate (V) and keyboard flying (W A S D / arrows)

**Graphics**
- Custom GLSL shaders: glowing stars with halos, planets lit by their nearest star, a black hole accretion disk, comet tails and pulsar beams
- Spiral arms of gas and dust, a starry background sky, motion trails and orbit rings
- MSAA anti-aliasing; glows fade as you approach so they never wash out the view

**Performance**
- Batched rendering of all glowing objects in a few draw calls
- Allocation-free pair loops (`split_at_mut`), squared-distance tests before any `sqrt`, and per-frame constants computed once

**Bug fixes from the 2D version**
- Two objects at exactly the same position (the galaxy center and its black hole) divided 0/0 in the collision code and turned into NaN on the first frame. This is now guarded.
- Pause only stopped expansion. It now freezes the whole universe, including the age counter.
- Collision correction pulled nearby objects *together*. It now only pushes overlapping objects apart.
- Selection used list indices, which changed whenever an object was removed. Every object now has a stable ID.
- The "click outside to close" area of the details window did not match its drawn size.

#### **Scientific Concepts (Simplified)**
**1. Gravitational Calculations**
The simulation uses Newton's law of universal gravitation, now in 3D, with a softening term for numerical stability:
```rust
let distance_sq = dx*dx + dy*dy + dz*dz + softening_length*softening_length;
```
The `softening_length` parameter prevents extreme forces at very close distances, maintaining simulation stability. Each pair is computed once and applied to both bodies (Newton's third law).

**2. Orbital Motion**
Planets orbit stars according to gravitational principles, with closer orbits moving faster (v = √(GM/r)). Each solar system has its own tilted orbital plane. The simulation uses a compressed timescale.

**3. Universe Expansion**
The model includes a very gradual Hubble-like expansion of distant objects (beyond ~1000 units from the center), with a small dark-energy factor that grows with age:
```
new_position = old_position × (1 + rate × distance) × dark_energy_factor
```
This results in minimal visible change over short observation periods.

**4. Collisions**
Close encounters are resolved with a 3D elastic collision (restitution 0.8) plus a small correction that only pushes overlapping objects apart.

**5. Deliberate Omissions**
To maintain stability and focus on orbital dynamics, the simulation does not include:
- Stellar evolution or death
- Aging processes
- Black hole formation (existing black holes are static objects)

#### **Time Scale**
- The simulation runs on a compressed timescale for observing long-term orbital patterns
- The age counter displays elapsed time (years, then millions of years)
- Hold Space to freeze time, hold Shift for slow motion

#### **System Architecture**
- **main.rs** - Primary control module handling user input and coordination.
- **camera.rs** - Smooth 3D orbit camera: rotation, zoom-to-cursor, panning and ray casting for picking.
- **universe.rs** - Initialization module that generates the galaxy, solar systems, nebulae and smaller bodies in 3D. It also handles object picking.
- **physics.rs** - Motion calculation module that updates velocities and resolves collisions.
- **gravity.rs** - Gravitational force calculation module implementing the softened attraction formula.
- **expansion.rs** - Module managing the gradual expansion of distant objects.
- **objects.rs** - Definition module for celestial object types, properties, stable IDs and visual characteristics.
- **rendering.rs** - Visualization module: shaders, glows, trails, orbits, the galaxy's dust, the background sky and the interface.

#### **Simulation Cycle (every frame)**
1. Process user input and camera movement
2. Update gravity, velocities and collisions
3. Apply expansion effects
4. Update positions and remove objects that drifted too far away
5. Render the 3D scene, then the interface
6. Repeat

#### **🎮 Controls**
| Key | Action |
|-----|--------|
| **Right Click + Drag** | Rotate camera |
| **Middle Click + Drag** | Pan camera |
| **W A S D / Arrows** | Move across the galactic plane |
| **Mouse Wheel** | Zoom in/out (towards the cursor) |
| **Left Click** | Select object |
| **Tab** | Show detailed info |
| **C** | Follow selected object |
| **V** | Auto-rotate camera |
| **R** | Reset view |
| **Space (hold)** | Pause time |
| **Shift (hold)** | Slow motion (30% speed) |
| **1, 2, 3** | Set expansion speed |
| **T / O / G / L** | Toggle trails / orbits / galaxy dust / links between nearby objects |
| **H** | Show/hide help |
| **F** | Toggle fullscreen |
| **ESC** | Close modals |

#### **Visual Information**
**Zoom Levels:**
- 1.5x: Planetary orbit rings appear
- 2.5x: Links between nearby objects appear (if enabled with L)

**Object Colors:**
- Red-Orange: Lower temperature stars
- Yellow: Medium temperature stars
- Blue-White: Higher temperature stars
- Varied colors: Planetary types (lit by their host star)
- Black with a glowing accretion disk: Black holes
- White-blue: Compact stars (neutron stars, pulsars, white dwarfs)
- Pink / blue / purple / orange / teal clouds: Nebulae

**Age Display:** Shows elapsed time since simulation start.

#### **Technical Considerations**
**Stability Design:**
- Softened gravity prevents calculation errors
- Speed limit and gentle collision resolution keep the system calm
- Objects that turn invalid (NaN) or drift beyond the simulation boundary are removed safely

**Scale Management:**
- Physical scales are significantly reduced for display
- Time compression allows observation of long-term orbital patterns

**Performance:**
- Calculates gravitational interactions between all object pairs (each pair once)
- Batched GPU rendering keeps the frame rate high, even with thousands of dust and sky particles

#### **Current Limitations**
- Occasional atypical orbital patterns
- Expansion effects are minimal by design
- Black holes do not grow by accretion; the accretion disk is a visual effect
- No audio components

#### **Design Philosophy**
This simulation was created to:
- Provide an accessible visualization of orbital mechanics
- Maintain stability during extended observation
- Demonstrate gravitational interactions
- Offer intuitive exploration through 3D navigation, zoom, pause, and selection features

It is intended as an educational visualization tool rather than a precise astrophysical model.

---

### **📁 Project Structure**
```
bigbang-simulator/
├── src/
│   ├── main.rs          # Main entry point and input handling
│   ├── camera.rs        # 3D orbit camera and ray casting
│   ├── universe.rs      # Universe creation, management and picking
│   ├── physics.rs       # Physics calculations and collisions
│   ├── gravity.rs       # Gravitational force calculations
│   ├── expansion.rs     # Cosmic expansion simulation
│   ├── objects.rs       # Celestial object definitions
│   └── rendering.rs     # Shaders, 3D graphics and UI rendering
├── Cargo.toml          # Project configuration
└── README.md           # This file
```

### **🧪 Scientific Concepts**
- **Newtonian Gravity:** F = G * (m1 * m2) / r²
- **Keplerian Orbits:** v = √(G * M / r)
- **Cosmic Expansion:** Hubble-like law, v ∝ distance
- **Orbital Stability:** Softening length prevents infinite forces
- **Collisions:** Impulse-based elastic collisions with restitution

### **Installation**
```bash
# Clone the repository
git clone https://github.com/itstaheri/bigbang-simulator.git
cd bigbang-simulator

# Build and run
cargo run --release
```

On Linux you may need: `sudo apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev`

### **🛠️ Building from Source**
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

### **📦 Cross-Platform Builds**
The simulator supports Windows, macOS, and Linux:
```bash
# Windows
cargo build --release --target x86_64-pc-windows-msvc

# macOS
cargo build --release --target x86_64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu
```

### **🧩 Dependencies**
- **macroquad:** Cross-platform graphics library (also provides random number generation and the 3D math)

### **🤝 Contributing**
Contributions are welcome! Please feel free to submit a Pull Request.
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

**Summary**
This simulation offers a simplified, stable representation of cosmic interactions in 3D, focusing on gravitational dynamics and orbital patterns in a controlled environment.

### 👨‍💻 Author
**Name:** [Ali Taheri]  
**Email:** itstaheri1@gmail.com  
**GitHub:** [@itstaheri](https://github.com/itstaheri)  
