### Big Bang Simulator 

**Note:** Welcome to my personal cosmic simulation! This is an interactive universe model that I designed and built for exploration, observation, and—above all—fun :)

#### **What You Can Experience**
**A Stable, Interactive Universe**
- Stars, planets, black holes, neutron stars, and other celestial objects
- Functional orbital mechanics
- Zoom capability to observe planetary motion around stars
- Select any object to view its properties and parameters

**Note:** This simulation is designed for stability and observation. Stars and planets do not undergo catastrophic events, and collisions are not part of the simulation. All objects interact through moderated gravitational forces.

#### **Scientific Concepts (Simplified)**
**1. Gravitational Calculations**
The simulation uses Newton's law of universal gravitation with a modification to ensure numerical stability:
```rust
let distance_sq = dx*dx + dy*dy + softening_length*softening_length;
```
The `softening_length` parameter prevents extreme forces at very close distances, maintaining simulation stability.

**2. Orbital Motion**
Planets orbit stars according to gravitational principles, with closer orbits moving faster. The simulation uses a compressed timescale: one second represents approximately 1,000 years.

**3. Universe Expansion**
The model includes a very gradual expansion effect, applied incrementally per frame:
```
new_position = old_position × (1 + 0.0000001)
```
This results in minimal visible change over short observation periods.

**4. Deliberate Omissions**
To maintain stability and focus on orbital dynamics, the simulation does not include:
- Stellar evolution or death
- Object collisions
- Aging processes
- Black hole formation (existing black holes are static objects)

#### **Time Scale**
- 1 simulation second ≈ 1,000 real years
- The age counter displays elapsed time in millions of years
- Pause function (Space key) allows freezing the simulation

#### **System Architecture**
- **main.rs** - Primary control module handling user input and coordination.
- **universe.rs** - Initialization module that generates galaxies, solar systems, and smaller celestial bodies.
- **physics.rs** - Motion calculation module that updates positions and velocities within set limits.
- **gravity.rs** - Gravitational force calculation module implementing the modified attraction formula.
- **expansion.rs** - Module managing the gradual expansion of distant objects.
- **objects.rs** - Definition module for celestial object types, properties, and visual characteristics.
- **rendering.rs** - Visualization module handling display elements, interface, and zoom-dependent rendering.

#### **Simulation Cycle (60 cycles per second)**
1. Process user input
2. Update physical calculations
3. Apply expansion effects
4. Render all visual elements
5. Repeat

#### **🎮 Controls**
| Key | Action |
|-----|--------|
| **Mouse Wheel** | Zoom in/out |
| **Right Click + Drag** | Pan camera |
| **Left Click** | Select object |
| **Space** | Pause/Resume time |
| **Shift** | Slow motion (30% speed) |
| **R / Middle Click** | Reset view |
| **1, 2, 3** | Set expansion speed |
| **F** | Toggle fullscreen |
| **Tab** | Show detailed info |
| **ESC** | Close modals |

#### **Visual Information**
**Zoom Levels:**
- 1.5x: Planetary orbits appear
- 2.0x: Binary star orbits appear
- 2.5x: Gravitational influence indicators appear
- 4.0x: Orbital position markers appear

**Object Colors:**
- Red-Orange: Lower temperature stars
- Yellow: Medium temperature stars
- Blue-White: Higher temperature stars
- Varied colors: Planetary types
- Black with purple highlight: Black hole representations
- White-blue: Compact stars

**Age Display:** Shows elapsed time in millions of years since simulation start.

#### **Technical Considerations**
**Stability Design:**
- Modified gravity prevents calculation errors
- No collisions or destructive events maintain observational continuity

**Scale Management:**
- Physical scales are significantly reduced for display
- Time compression allows observation of long-term orbital patterns

**Performance:**
- Calculates gravitational interactions between all objects
- Maintains performance through optimized calculations
- Designed to run smoothly with up to 100 objects

#### **Current Limitations**
- Occasional atypical orbital patterns
- View adjustments during interaction may cause visual discontinuity
- Expansion effects are minimal by design
- Black holes are visual elements without accretion dynamics
- No audio components

#### **Design Philosophy**
This simulation was created to:
- Provide an accessible visualization of orbital mechanics
- Maintain stability during extended observation
- Demonstrate gravitational interactions
- Offer intuitive exploration through zoom, pause, and selection features

It is intended as an educational visualization tool rather than a precise astrophysical model.

---

### **📁 Project Structure**
```
bigbang-simulator/
├── src/
│   ├── main.rs          # Main entry point and input handling
│   ├── universe.rs      # Universe creation and management
│   ├── physics.rs       # Physics calculations and updates
│   ├── gravity.rs       # Gravitational force calculations
│   ├── expansion.rs     # Cosmic expansion simulation
│   ├── objects.rs       # Celestial object definitions
│   └── rendering.rs     # Graphics and UI rendering
├── Cargo.toml          # Project configuration
└── README.md           # This file
```

### **🧪 Scientific Concepts**
- **Newtonian Gravity:** F = G * (m1 * m2) / r²
- **Keplerian Orbits:** v = √(G * M / r)
- **Cosmic Expansion:** Positions scale with 1 + expansion_rate
- **Orbital Stability:** Softening length prevents infinite forces

### **Installation**
```bash
# Clone the repository
git clone https://github.com/yourusername/bigbang-simulator.git
cd bigbang-simulator

# Build and run
cargo run --release
```

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
- **macroquad:** Cross-platform graphics library
- **rand:** Random number generation

### **🤝 Contributing**
Contributions are welcome! Please feel free to submit a Pull Request.
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

**Summary**
This simulation offers a simplified, stable representation of cosmic interactions, focusing on gravitational dynamics and orbital patterns in a controlled environment.

### 👨‍💻 Author
**Name:** [Ali Taheri]  
**Email:** itstaheri1@gmail.com  
**GitHub:** [@itstaheri](https://github.com/itstaheri)  

