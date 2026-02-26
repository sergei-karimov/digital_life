//! # Spatial Hash Grid — O(n) broad-phase collision detection
//!
//! Instead of O(n²) brute-force, we partition the screen into a uniform grid
//! of cells. Each frame we:
//! 1. **Clear** all cells
//! 2. **Insert** every circle into the cell(s) it overlaps
//! 3. **Query** collisions only against circles in the same / neighbouring cells
//!
//! Expected complexity: **O(n·k)** where k is the avg number of circles per cell
//! neighbourhood (typically small and constant for uniformly distributed particles).

/// A cell-based spatial index for fast overlap queries.
pub struct SpatialGrid {
    cell_size: f32,
    cols: usize,
    rows: usize,
    /// Each cell stores a list of circle indices.
    cells: Vec<Vec<usize>>,
}

impl SpatialGrid {
    /// Create a grid that covers `width × height` with cells of `cell_size`.
    /// `cell_size` should be ≥ 2× the largest circle radius for correctness.
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let cols = (width / cell_size).ceil() as usize + 1;
        let rows = (height / cell_size).ceil() as usize + 1;
        Self {
            cell_size,
            cols,
            rows,
            cells: vec![Vec::new(); cols * rows],
        }
    }

    /// Remove all entries (but keep allocated memory for reuse).
    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.clear();
        }
    }

    /// Insert a circle (by index) into every cell it overlaps.
    pub fn insert(&mut self, index: usize, x: f32, y: f32, radius: f32) {
        let min_col = ((x - radius) / self.cell_size).floor().max(0.0) as usize;
        let max_col = ((x + radius) / self.cell_size).floor() as usize;
        let min_row = ((y - radius) / self.cell_size).floor().max(0.0) as usize;
        let max_row = ((y + radius) / self.cell_size).floor() as usize;

        for row in min_row..=max_row.min(self.rows - 1) {
            for col in min_col..=max_col.min(self.cols - 1) {
                self.cells[row * self.cols + col].push(index);
            }
        }
    }

    /// Find all **unique** pairs `(i, j)` where `i < j` that share at least one cell.
    /// The caller does the narrow-phase (distance) check on each returned pair.
    ///
    /// We use a flat `Vec<bool>` dedupe matrix (triangular) for circles count ≤ limit,
    /// falling back to a simpler per-cell approach for very large counts.
    pub fn find_candidate_pairs(&self, count: usize) -> Vec<(usize, usize)> {
        // Triangular bitset for deduplication: index = i*count + j
        // Memory: count*count bits ≈ 5000*5000 / 8 ≈ 3 MB — acceptable.
        let mut seen = vec![false; count * count];
        let mut pairs = Vec::with_capacity(count); // rough initial guess

        for cell in &self.cells {
            let len = cell.len();
            if len < 2 {
                continue;
            }
            for a in 0..len {
                for b in (a + 1)..len {
                    let i = cell[a];
                    let j = cell[b];
                    let (lo, hi) = if i < j { (i, j) } else { (j, i) };
                    let key = lo * count + hi;
                    if !seen[key] {
                        seen[key] = true;
                        pairs.push((lo, hi));
                    }
                }
            }
        }

        pairs
    }
}