//! # Movement Engine Module
//!
//! This module defines the movement behavior for circles.
//! It's designed as a **strategy pattern** — implement [`MovementEngine`] trait
//! to create new movement behaviors without touching the rest of the codebase.
//!
//! ## Adding a new engine
//! 1. Create a struct implementing [`MovementEngine`]
//! 2. Add a variant to [`EngineKind`]
//! 3. Register it in [`EngineKind::create`]

use rand::Rng;

// ── Public Trait ──────────────────────────────────────────────────────────────

/// Core trait for any movement strategy.
///
/// Each circle owns a boxed `dyn MovementEngine`. Every frame the simulation
/// calls [`update`] which returns the (dx, dy) delta to apply to the circle's
/// position. The engine receives the current position and the screen bounds so
/// it can implement boundary-aware behaviors (bounce, wrap, etc.).
pub trait MovementEngine: Send + Sync {
    /// Compute frame delta. Returns `(dx, dy)`.
    fn update(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        screen_w: f32,
        screen_h: f32,
        dt: f32,
    ) -> (f32, f32);

    /// Human-readable name (for debug overlays).
    fn name(&self) -> &'static str;
}

// ── Engine Registry ──────────────────────────────────────────────────────────

/// Available engine kinds. Extend this enum to add more behaviors.
#[derive(Clone, Copy, Debug)]
pub enum EngineKind {
    /// Moves in a straight line, bounces off walls.
    LinearBounce,
    /// Wanders randomly, changing direction over time.
    RandomWalk,
    /// Orbits around a slowly-drifting anchor point.
    Orbital,
}

impl EngineKind {
    /// Factory: create a concrete engine with randomised initial parameters.
    pub fn create(self, rng: &mut impl Rng) -> Box<dyn MovementEngine> {
        match self {
            EngineKind::LinearBounce => Box::new(LinearBounceEngine::new(rng)),
            EngineKind::RandomWalk => Box::new(RandomWalkEngine::new(rng)),
            EngineKind::Orbital => Box::new(OrbitalEngine::new(rng)),
        }
    }

    /// Pick a random engine kind.
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0..3) {
            0 => EngineKind::LinearBounce,
            1 => EngineKind::RandomWalk,
            _ => EngineKind::Orbital,
        }
    }
}

// ── 1. Linear Bounce ─────────────────────────────────────────────────────────

/// Moves at constant velocity, reflects off screen edges.
pub struct LinearBounceEngine {
    vx: f32,
    vy: f32,
}

impl LinearBounceEngine {
    pub fn new(rng: &mut impl Rng) -> Self {
        let speed = rng.gen_range(40.0..160.0);
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        Self {
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
        }
    }
}

impl MovementEngine for LinearBounceEngine {
    fn update(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        screen_w: f32,
        screen_h: f32,
        dt: f32,
    ) -> (f32, f32) {
        let nx = x + self.vx * dt;
        let ny = y + self.vy * dt;

        if nx - radius < 0.0 || nx + radius > screen_w {
            self.vx = -self.vx;
        }
        if ny - radius < 0.0 || ny + radius > screen_h {
            self.vy = -self.vy;
        }

        (self.vx * dt, self.vy * dt)
    }

    fn name(&self) -> &'static str {
        "LinearBounce"
    }
}

// ── 2. Random Walk ───────────────────────────────────────────────────────────

/// Smoothly wanders by rotating its heading with Perlin-like noise.
pub struct RandomWalkEngine {
    angle: f32,
    speed: f32,
    turn_rate: f32,
    turn_timer: f32,
    turn_target: f32,
}

impl RandomWalkEngine {
    pub fn new(rng: &mut impl Rng) -> Self {
        Self {
            angle: rng.gen_range(0.0..std::f32::consts::TAU),
            speed: rng.gen_range(30.0..120.0),
            turn_rate: rng.gen_range(1.0..4.0),
            turn_timer: 0.0,
            turn_target: rng.gen_range(-1.5..1.5),
        }
    }
}

impl MovementEngine for RandomWalkEngine {
    fn update(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        screen_w: f32,
        screen_h: f32,
        dt: f32,
    ) -> (f32, f32) {
        // Periodically pick a new turn target
        self.turn_timer += dt;
        if self.turn_timer > 0.8 {
            self.turn_timer = 0.0;
            // Use a simple LCG-style randomness from position to avoid storing rng
            let seed = ((x * 1000.0 + y * 7.0) as u32).wrapping_mul(2654435761);
            let norm = (seed as f32) / (u32::MAX as f32);
            self.turn_target = (norm - 0.5) * 3.0;
        }

        // Steer towards turn target
        let diff = self.turn_target - self.angle;
        self.angle += diff.signum() * self.turn_rate.min(diff.abs()) * dt;

        let dx = self.angle.cos() * self.speed * dt;
        let dy = self.angle.sin() * self.speed * dt;

        // Soft bounce: steer away from walls
        if x + dx - radius < 0.0 || x + dx + radius > screen_w {
            self.angle = std::f32::consts::PI - self.angle;
        }
        if y + dy - radius < 0.0 || y + dy + radius > screen_h {
            self.angle = -self.angle;
        }

        let dx = self.angle.cos() * self.speed * dt;
        let dy = self.angle.sin() * self.speed * dt;
        (dx, dy)
    }

    fn name(&self) -> &'static str {
        "RandomWalk"
    }
}

// ── 3. Orbital ───────────────────────────────────────────────────────────────

/// Orbits around a drifting anchor point.
pub struct OrbitalEngine {
    anchor_x: f32,
    anchor_y: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    phase: f32,
    drift_vx: f32,
    drift_vy: f32,
}

impl OrbitalEngine {
    pub fn new(rng: &mut impl Rng) -> Self {
        Self {
            anchor_x: 0.0, // will be initialised on first update
            anchor_y: 0.0,
            orbit_radius: rng.gen_range(10.0..50.0),
            orbit_speed: rng.gen_range(1.5..5.0),
            phase: rng.gen_range(0.0..std::f32::consts::TAU),
            drift_vx: rng.gen_range(-20.0..20.0),
            drift_vy: rng.gen_range(-20.0..20.0),
        }
    }
}

impl MovementEngine for OrbitalEngine {
    fn update(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        screen_w: f32,
        screen_h: f32,
        dt: f32,
    ) -> (f32, f32) {
        // Lazy init anchor to current position
        if self.anchor_x == 0.0 && self.anchor_y == 0.0 {
            self.anchor_x = x;
            self.anchor_y = y;
        }

        // Drift the anchor
        self.anchor_x += self.drift_vx * dt;
        self.anchor_y += self.drift_vy * dt;

        // Bounce anchor off walls
        if self.anchor_x < radius || self.anchor_x > screen_w - radius {
            self.drift_vx = -self.drift_vx;
            self.anchor_x = self.anchor_x.clamp(radius, screen_w - radius);
        }
        if self.anchor_y < radius || self.anchor_y > screen_h - radius {
            self.drift_vy = -self.drift_vy;
            self.anchor_y = self.anchor_y.clamp(radius, screen_h - radius);
        }

        self.phase += self.orbit_speed * dt;

        let target_x = self.anchor_x + self.phase.cos() * self.orbit_radius;
        let target_y = self.anchor_y + self.phase.sin() * self.orbit_radius;

        (target_x - x, target_y - y)
    }

    fn name(&self) -> &'static str {
        "Orbital"
    }
}