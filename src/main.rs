mod circle;
mod collision;
mod engine;

use circle::Circle;
use collision::SpatialGrid;
use engine::EngineKind;

use rand::Rng;
use raylib::prelude::*;

// ── Configuration ────────────────────────────────────────────────────────────

/// Number of circles in the simulation.
const CIRCLE_COUNT: usize = 3000;
/// Min / max radius of a spawned circle.
const RADIUS_MIN: f32 = 1.0;
const RADIUS_MAX: f32 = 1.5;
/// Colour when two circles collide (bright highlight).
const COLLISION_COLOR: Color = Color::new(255, 50, 80, 255); // vivid red-pink
/// Background colour.
const BG_COLOR: Color = Color::new(10, 10, 18, 255); // near-black

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Generate a random soft colour for a circle's base state.
fn random_base_color(rng: &mut impl Rng) -> Color {
    Color::new(
        rng.gen_range(40..180),
        rng.gen_range(80..220),
        rng.gen_range(120..255),
        200,
    )
}

fn spawn_circles(count: usize, screen_w: f32, screen_h: f32) -> Vec<Circle> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            let radius = rng.gen_range(RADIUS_MIN..RADIUS_MAX);
            let x = rng.gen_range(radius..(screen_w - radius));
            let y = rng.gen_range(radius..(screen_h - radius));
            let color = random_base_color(&mut rng);
            let kind = EngineKind::random(&mut rng);
            let eng = kind.create(&mut rng);
            Circle::new(x, y, radius, color, eng)
        })
        .collect()
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    // ── Window init (fullscreen) ─────────────────────────────────────────
    let (mut rl, thread) = raylib::init()
        .title("Circles — Spatial Hash Collisions")
        .size(1920, 1080)
        .vsync()
        .build();

    let screen_w = rl.get_screen_width() as f32;
    let screen_h = rl.get_screen_height() as f32;

    rl.set_target_fps(60);

    // ── Spawn circles ────────────────────────────────────────────────────
    let mut circles = spawn_circles(CIRCLE_COUNT, screen_w, screen_h);

    // ── Spatial grid ─────────────────────────────────────────────────────
    // Cell size = 2× max radius → guarantees any overlapping pair shares a cell.
    let cell_size = RADIUS_MAX * 4.0;
    let mut grid = SpatialGrid::new(screen_w, screen_h, cell_size);

    // Collision count for the HUD
    let mut collision_count: usize = 0;

    // ── Main loop ────────────────────────────────────────────────────────
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // ── 1. Movement update ───────────────────────────────────────────
        for c in circles.iter_mut() {
            let (dx, dy) = c.engine.update(c.x, c.y, c.radius, screen_w, screen_h, dt);
            c.x += dx;
            c.y += dy;

            // Hard clamp to screen bounds (safety net)
            c.x = c.x.clamp(c.radius, screen_w - c.radius);
            c.y = c.y.clamp(c.radius, screen_h - c.radius);
        }

        // ── 2. Broad-phase: build spatial grid ───────────────────────────
        grid.clear();
        for (i, c) in circles.iter().enumerate() {
            grid.insert(i, c.x, c.y, c.radius);
        }

        // ── 3. Reset collision flags ─────────────────────────────────────
        for c in circles.iter_mut() {
            c.colliding = false;
        }

        // ── 4. Narrow-phase: check candidate pairs ──────────────────────
        let pairs = grid.find_candidate_pairs(circles.len());
        collision_count = 0;

        for &(i, j) in &pairs {
            let (ci_x, ci_y, ci_r) = (circles[i].x, circles[i].y, circles[i].radius);
            let (cj_x, cj_y, cj_r) = (circles[j].x, circles[j].y, circles[j].radius);

            let dx = ci_x - cj_x;
            let dy = ci_y - cj_y;
            let dist_sq = dx * dx + dy * dy;
            let min_dist = ci_r + cj_r;

            if dist_sq < min_dist * min_dist {
                circles[i].colliding = true;
                circles[j].colliding = true;
                collision_count += 1;
            }
        }

        // ── 5. Update render colour ─────────────────────────────────────
        for c in circles.iter_mut() {
            c.render_color = if c.colliding {
                COLLISION_COLOR
            } else {
                c.base_color
            };
        }

        // ── 6. Draw ─────────────────────────────────────────────────────
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BG_COLOR);

        for c in &circles {
            d.draw_circle(
                c.x as i32,
                c.y as i32,
                c.radius,
                c.render_color,
            );
        }

        // HUD overlay
        d.draw_fps(10, 10);
        d.draw_text(
            &format!("Circles: {}  |  Collisions: {}  |  Pairs checked: {}",
                      circles.len(), collision_count, pairs.len()),
            10,
            40,
            20,
            Color::RAYWHITE,
        );
        d.draw_text(
            "Press ESC to exit",
            10,
            screen_h as i32 - 30,
            18,
            Color::new(120, 120, 120, 180),
        );
    }
}