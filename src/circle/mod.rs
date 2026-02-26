//! Circle entity definition.

use raylib::color::Color;

use crate::engine::MovementEngine;

/// A single circle particle in the simulation.
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,

    /// Base (non-colliding) color.
    pub base_color: Color,
    /// Current render color (may differ during collision highlight).
    pub render_color: Color,

    /// Pluggable movement strategy — see [`crate::engine`].
    pub engine: Box<dyn MovementEngine>,

    /// True this frame if overlapping with another circle.
    pub colliding: bool,
}

impl Circle {
    pub fn new(
        x: f32,
        y: f32,
        radius: f32,
        base_color: Color,
        engine: Box<dyn MovementEngine>,
    ) -> Self {
        Self {
            x,
            y,
            radius,
            base_color,
            render_color: base_color,
            engine,
            colliding: false,
        }
    }
}