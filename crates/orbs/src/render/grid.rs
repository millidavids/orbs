//! The cell grid, as one mesh.
//!
//! DESIGN.md §4 rules out the `bevy_text`/`bevy_ui` path outright — it "will not
//! hold a full grid redrawing under a real-time siege" — and names the
//! alternative: *"single mesh with glyph atlas and instanced quads, or a custom
//! material driven by a cell-index texture."*
//!
//! This is the first of those. Every visible cell contributes four vertices to
//! one mesh, so the whole screen is **one draw call** regardless of grid size.
//!
//! # Where the buffers live
//!
//! In the mesh, and nowhere else. Each attribute's `Vec` is taken out, refilled,
//! and handed back, so a steady-state frame allocates nothing. An earlier version
//! kept a parallel `GridBuffers` and *cloned* it into the mesh every frame —
//! ~900 kB of allocation at the 120×45 grid, which defeated the reuse it was
//! written for, and which the reuse test could not see because it only inspected
//! the staging copy.
//!
//! # Why no custom shader
//!
//! Each vertex carries its own colour, and Bevy's stock `ColorMaterial`
//! multiplies the sampled texel by it. With the atlas storing white RGB and
//! coverage in alpha ([`super::atlas`]), `(1,1,1,coverage) * (r,g,b,1)` is
//! exactly "this glyph, in this cell's colour" — with no WGSL of our own. The
//! CRT port (§4) adds a post-process pass later; that is a separate stage and
//! does not change this one.
//!
//! # Coordinates
//!
//! The grid is centred on the origin with cell `(0, 0)` at the top left, because
//! `orbs-render` counts rows downward and Bevy's 2D camera has +Y up. Cell size
//! is an integer number of pixels (§4), so glyphs land on whole pixels and the
//! bitmap stays crisp.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use orbs_render::{CELL_HEIGHT, CELL_WIDTH, Frame, Style, cp437};

use super::atlas;
use super::palette::Phosphor;

/// The glyph the caret is drawn as. A solid block, as terminals have always
/// drawn it — we have no inverse video to fall back on.
const CARET: char = '█';

/// Vertex data on its way into the mesh.
struct Geometry {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colours: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Geometry {
    /// Reclaim the mesh's own buffers, keeping their capacity.
    fn reclaim(mesh: &mut Mesh) -> Self {
        let positions = match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => core::mem::take(values),
            _ => Vec::new(),
        };
        let uvs = match mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
            Some(VertexAttributeValues::Float32x2(values)) => core::mem::take(values),
            _ => Vec::new(),
        };
        let colours = match mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(values)) => core::mem::take(values),
            _ => Vec::new(),
        };
        let indices = match mesh.indices_mut() {
            Some(Indices::U32(values)) => core::mem::take(values),
            _ => Vec::new(),
        };

        let mut geometry = Self {
            positions,
            uvs,
            colours,
            indices,
        };
        geometry.clear();
        geometry
    }

    fn clear(&mut self) {
        self.positions.clear();
        self.uvs.clear();
        self.colours.clear();
        self.indices.clear();
    }

    /// One cell: four vertices, two triangles, wound counter-clockwise.
    fn push_cell(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        colour: [f32; 4],
        uv: (f32, f32, f32, f32),
    ) {
        let (u0, v0, u1, v1) = uv;
        let base = u32::try_from(self.positions.len()).unwrap_or(0);

        self.positions.extend_from_slice(&[
            [x, y, 0.0],
            [x + width, y, 0.0],
            [x + width, y - height, 0.0],
            [x, y - height, 0.0],
        ]);
        self.uvs
            .extend_from_slice(&[[u0, v0], [u1, v0], [u1, v1], [u0, v1]]);
        self.colours.extend_from_slice(&[colour; 4]);
        self.indices
            .extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    }

    /// Hand the buffers back to the mesh. Moves, never copies.
    fn commit(self, mesh: &mut Mesh) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colours);
        mesh.insert_indices(Indices::U32(self.indices));
    }
}

/// Rebuild `mesh` from `frame`.
///
/// Blank cells contribute nothing: a space is the commonest glyph on screen by a
/// wide margin, and a quad that samples a fully transparent texel is pure cost.
///
/// `showing` is the caret's blink phase — see [`Blink`](super::blink::Blink).
///
/// # One world unit is one **virtual** pixel
///
/// A cell is always 8×16 units, whatever the window. Fitting that to real pixels
/// is the camera's job — `ScalingMode::AutoMin`, set in `shell::screen`'s
/// `spawn_camera`, letterboxes the whole picture — so there is no scale factor
/// here at all. This used to take one, back when the grid was derived from the
/// window and the cell had to be grown to match.
pub(crate) fn build(frame: &Frame, theme: &Phosphor, showing: bool, mesh: &mut Mesh) {
    let mut geometry = Geometry::reclaim(mesh);

    let grid = frame.size();
    let cell_width = f32::from(CELL_WIDTH);
    let cell_height = f32::from(CELL_HEIGHT);
    let left = -(f32::from(grid.cols) * cell_width) / 2.0;
    let top = (f32::from(grid.rows) * cell_height) / 2.0;

    let position = |column: f32, row: f32| (left + column * cell_width, top - row * cell_height);

    // A region the `Frame` marked for double-size drawing — the prompt. Its
    // cells are skipped here and drawn below at 2×, so one row of them fills two
    // rows and twice the columns.
    let magnified = frame.magnified();
    let inside = |column: usize, row: usize| {
        magnified.is_some_and(|area| {
            let (column, row) = (
                u16::try_from(column).unwrap_or(u16::MAX),
                u16::try_from(row).unwrap_or(u16::MAX),
            );
            row == area.row && column >= area.col && column < area.right()
        })
    };

    for (row, cells) in frame.rows().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            if cell.is_blank() || inside(column, row) {
                continue;
            }
            let Some(index) = cp437::cp437_index(cell.glyph) else {
                continue;
            };
            let (x, y) = position(grid_offset(column), grid_offset(row));
            // The material's colour family, if this cell falls in a tinted
            // region. Per *region* rather than per cell — see `Frame::tint_at`
            // for why a `Cell` does not carry one.
            let at = orbs_render::Pos::new(
                u16::try_from(column).unwrap_or(u16::MAX),
                u16::try_from(row).unwrap_or(u16::MAX),
            );
            geometry.push_cell(
                x,
                y,
                cell_width,
                cell_height,
                theme
                    .resolve_tinted(cell.style, frame.tint_at(at), frame.lit_at(at))
                    .to_linear()
                    .to_f32_array(),
                atlas::uv(index, cell.style.presentation),
            );
        }
    }

    // The magnified pass. Same glyphs, same atlas, quads twice the size — which
    // is the whole trick: no second cell ratio, no second font, and the grid
    // stays one grid.
    if let Some(area) = magnified {
        for offset in 0..area.cols {
            let column = area.col.saturating_add(offset);
            let Some(cell) = frame.cell(orbs_render::Pos::new(column, area.row)) else {
                continue;
            };
            if cell.is_blank() {
                continue;
            }
            let Some(index) = cp437::cp437_index(cell.glyph) else {
                continue;
            };
            // Anchored at the region's own origin and stepping two cells per
            // glyph, so the run stays flush with the left edge rather than
            // drifting right by its own magnification.
            let (x, y) = position(
                grid_offset(usize::from(area.col)) + f32::from(offset) * 2.0,
                grid_offset(usize::from(area.row)),
            );
            geometry.push_cell(
                x,
                y,
                cell_width * 2.0,
                cell_height * 2.0,
                theme.resolve(cell.style).to_linear().to_f32_array(),
                atlas::uv(index, cell.style.presentation),
            );
        }
    }

    // The caret. `Frame` carries it, so a frontend that dropped it would show a
    // different screen from one that did not — which is the disagreement
    // architectural rule 2 exists to prevent.
    //
    // `showing` is the blink phase, and blinking is legitimate frontend
    // enrichment: the caret's *position* is in the Frame, and a blink adds no
    // information a static block does not already carry. `orbs-tui` gets the
    // terminal's own cursor and is none the poorer.
    if showing
        && let Some(caret) = frame.cursor()
        && let Some(index) = cp437::cp437_index(CARET)
    {
        // The caret magnifies with the text it trails. A full-size block beside
        // a double-size prompt reads as a rendering fault, and it is the one
        // glyph on screen whose whole job is saying *here*.
        let big = magnified.is_some_and(|area| {
            caret.row == area.row && caret.col >= area.col && caret.col <= area.right()
        });
        let (x, y) = match magnified.filter(|_| big) {
            Some(area) => position(
                f32::from(area.col) + f32::from(caret.col.saturating_sub(area.col)) * 2.0,
                f32::from(area.row),
            ),
            None => position(f32::from(caret.col), f32::from(caret.row)),
        };
        let (width, height) = if big {
            (cell_width * 2.0, cell_height * 2.0)
        } else {
            (cell_width, cell_height)
        };
        geometry.push_cell(
            x,
            y,
            width,
            height,
            theme.resolve(Style::BRIGHT).to_linear().to_f32_array(),
            atlas::uv(index, orbs_render::Presentation::Plain),
        );

        // **Reverse video.** The caret used to sit one past the end of the line,
        // always on a blank, so a solid block was fine. It can sit mid-line now,
        // and a block drawn over a character hides the character — blinking it in
        // and out, in the one place the player is looking.
        //
        // So the glyph underneath is redrawn on top of the block in the tube's
        // own black, which is what a terminal does and what the `CARET` comment
        // meant by *"we have no inverse video to fall back on"*. Entirely a
        // frontend job: the caret is a quad rather than a cell, so no per-cell
        // inverse flag is needed and §19's *"what a cell holds"* stands.
        if let Some(under) = frame.cell(caret)
            && !under.is_blank()
            && let Some(glyph) = cp437::cp437_index(under.glyph)
        {
            geometry.push_cell(
                x,
                y,
                width,
                height,
                Color::BLACK.to_linear().to_f32_array(),
                atlas::uv(glyph, under.style.presentation),
            );
        }
    }

    geometry.commit(mesh);
}

/// The mesh the grid starts with, before anything has been drawn.
///
/// **Not zero-vertex.** Bevy 0.19's slab allocator answers a zero-length vertex
/// buffer with `use-after-free: attempted to copy element data for an
/// unallocated key`, and until the boot sequence existed nothing ever held an
/// empty screen long enough for one to reach the GPU — the first frame always
/// had the tower's report on it. It carries one degenerate triangle instead:
/// three coincident vertices at the origin, fully transparent, which rasterises
/// to no pixels and keeps the buffer allocated.
///
/// See `render::plugin::rasterise` for the other half — a blank screen must not
/// rebuild the mesh at all, or this happens sixty times a second.
pub(crate) fn empty_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 3]);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    mesh
}

/// `usize` grid coordinate to pixels, without a lossy cast lint at every site.
fn grid_offset(index: usize) -> f32 {
    /// The furthest a cell coordinate can be, when one does not fit a `u16`.
    const FURTHEST: f32 = u16::MAX as f32;

    u16::try_from(index).map_or(FURTHEST, f32::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::{GridSize, Pos, Span};

    use crate::render::palette::MUTED_VIOLET;

    /// Read the mesh back, since the mesh *is* the buffer now.
    fn positions(mesh: &Mesh) -> Vec<[f32; 3]> {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values.clone(),
            _ => Vec::new(),
        }
    }

    fn colours(mesh: &Mesh) -> Vec<[f32; 4]> {
        match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(values)) => values.clone(),
            _ => Vec::new(),
        }
    }

    fn index_count(mesh: &Mesh) -> usize {
        match mesh.indices() {
            Some(Indices::U32(values)) => values.len(),
            _ => 0,
        }
    }

    fn quads(mesh: &Mesh) -> usize {
        positions(mesh).len() / 4
    }

    fn frame_with(text: &str, cols: u16, rows: u16) -> Frame {
        let mut frame = Frame::new(GridSize::new(cols, rows));
        frame
            .painter(frame.area())
            .span(Pos::ORIGIN, &Span::new(text));
        frame
    }

    fn built(frame: &Frame) -> Mesh {
        let mut mesh = empty_mesh();
        build(frame, &MUTED_VIOLET, true, &mut mesh);
        mesh
    }

    #[test]
    fn one_quad_per_visible_cell() {
        assert_eq!(quads(&built(&frame_with("abc", 10, 1))), 3);
    }

    #[test]
    fn blank_cells_cost_nothing() {
        // A space is the commonest glyph on screen; drawing a transparent quad
        // for every one of them would be most of the grid.
        assert_eq!(quads(&built(&Frame::new(GridSize::new(80, 22)))), 0);
        assert_eq!(
            quads(&built(&frame_with("a b", 80, 22))),
            2,
            "the space between should not be drawn"
        );
    }

    #[test]
    fn a_magnified_row_is_drawn_once_at_double_size() {
        // The prompt. One row of cells, quads twice the size — no second cell
        // ratio and no second font, which is what keeps the grid one grid.
        let mut frame = frame_with("abc", 10, 2);
        let plain = quads(&built(&frame));

        frame.set_magnified(Some(orbs_render::Rect::new(0, 0, 3, 1)));
        let magnified = built(&frame);

        assert_eq!(
            quads(&magnified),
            plain,
            "a magnified cell was drawn twice, or dropped"
        );

        // Twice as wide and twice as tall as a normal cell.
        let corners = positions(&magnified);
        let width = corners[1][0] - corners[0][0];
        let height = corners[0][1] - corners[3][1];
        assert!(
            (width - f32::from(orbs_render::CELL_WIDTH) * 2.0).abs() < f32::EPSILON,
            "magnified width was {width}"
        );
        assert!(
            (height - f32::from(orbs_render::CELL_HEIGHT) * 2.0).abs() < f32::EPSILON,
            "magnified height was {height}"
        );
    }

    #[test]
    fn a_magnified_run_steps_two_cells_a_glyph() {
        // Stepping one would overlap each glyph with the last by half, which
        // reads as a smear rather than as text.
        let mut frame = frame_with("ab", 10, 2);
        frame.set_magnified(Some(orbs_render::Rect::new(0, 0, 2, 1)));
        let corners = positions(&built(&frame));

        let first = corners[0][0];
        let second = corners[4][0];
        assert!(
            (second - first - f32::from(orbs_render::CELL_WIDTH) * 2.0).abs() < f32::EPSILON,
            "glyphs stepped {} apart",
            second - first
        );
    }

    #[test]
    fn the_caret_is_drawn() {
        // `Frame` carries a cursor. A frontend that dropped it would show a
        // different screen from one that did not.
        let mut frame = frame_with("ab", 10, 1);
        let without = quads(&built(&frame));

        frame.set_cursor(Some(Pos::new(3, 0)));
        assert_eq!(quads(&built(&frame)), without + 1);
    }

    #[test]
    fn a_caret_outside_the_grid_draws_nothing() {
        let mut frame = frame_with("ab", 4, 1);
        frame.set_cursor(Some(Pos::new(99, 99)));
        // `Frame::set_cursor` refuses positions off the grid, so there is
        // nothing extra to draw.
        assert_eq!(quads(&built(&frame)), 2);
    }

    #[test]
    fn the_grid_is_centred_on_the_origin() {
        let mesh = built(&frame_with("a", 1, 1));
        let points = positions(&mesh);
        let xs: Vec<f32> = points.iter().map(|p| p[0]).collect();
        let ys: Vec<f32> = points.iter().map(|p| p[1]).collect();

        let width = f32::from(CELL_WIDTH);
        let height = f32::from(CELL_HEIGHT);
        assert!((xs.iter().copied().fold(f32::MAX, f32::min) + width / 2.0).abs() < 0.01);
        assert!((ys.iter().copied().fold(f32::MIN, f32::max) - height / 2.0).abs() < 0.01);
    }

    #[test]
    fn cell_zero_is_the_top_left() {
        // orbs-render counts rows downward; Bevy's 2D camera has +Y up. Getting
        // this backwards renders the screen upside down.
        let mut frame = Frame::new(GridSize::new(2, 2));
        frame
            .painter(frame.area())
            .span(Pos::new(0, 0), &Span::new("a"));
        let top = positions(&built(&frame))[0][1];

        let mut frame = Frame::new(GridSize::new(2, 2));
        frame
            .painter(frame.area())
            .span(Pos::new(0, 1), &Span::new("a"));
        let below = positions(&built(&frame))[0][1];

        assert!(top > below, "row 0 should sit above row 1");
    }

    #[test]
    fn a_cell_is_always_one_bitmap_cell_of_world() {
        // The mesh is emitted in **virtual** pixels and the camera scales it, so
        // there is no scale factor here to get wrong. This used to assert that
        // an integer scale multiplied the cell exactly; the property that
        // replaced it is that the cell never changes at all.
        let positions = positions(&built(&frame_with("a", 4, 1)));
        let width = positions[1][0] - positions[0][0];
        let height = positions[0][1] - positions[2][1];

        assert!((width - f32::from(CELL_WIDTH)).abs() < 0.001, "{width}");
        assert!((height - f32::from(CELL_HEIGHT)).abs() < 0.001, "{height}");
    }

    #[test]
    fn every_quad_has_four_vertices_and_six_indices() {
        let mesh = built(&frame_with("hello", 10, 1));
        assert_eq!(positions(&mesh).len(), quads(&mesh) * 4);
        assert_eq!(colours(&mesh).len(), positions(&mesh).len());
        assert_eq!(index_count(&mesh), quads(&mesh) * 6);
    }

    /// The bug the previous version shipped with.
    ///
    /// It kept a staging buffer, cloned it into the mesh every frame, and
    /// asserted only that the *staging* buffer kept its capacity — so ~900 kB of
    /// per-frame allocation passed a test named for reuse.
    #[test]
    fn rebuilding_allocates_nothing() {
        let mut mesh = empty_mesh();
        let frame = frame_with("something reasonably long", 40, 1);

        build(&frame, &MUTED_VIOLET, true, &mut mesh);
        let capacity = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values.capacity(),
            _ => 0,
        };
        assert!(capacity > 0);

        for _ in 0..8 {
            build(&frame, &MUTED_VIOLET, true, &mut mesh);
        }

        let after = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values.capacity(),
            _ => 0,
        };
        assert_eq!(after, capacity, "the mesh reallocated between frames");
    }

    #[test]
    fn style_reaches_the_vertex_colours() {
        let mut frame = Frame::new(GridSize::new(4, 1));
        frame
            .painter(frame.area())
            .span(Pos::ORIGIN, &Span::new("x").with_style(Style::DANGER));
        let expected = MUTED_VIOLET
            .resolve(Style::DANGER)
            .to_linear()
            .to_f32_array();
        assert_eq!(colours(&built(&frame))[0], expected);
    }

    /// §4 warns that a full grid redrawing under a real-time siege is what the
    /// `bevy_text` path could not hold. This is the number that says whether the
    /// single-mesh path can.
    ///
    /// The bound is generous because tests run unoptimised; a release build is
    /// roughly an order of magnitude faster. It is here to catch a regression in
    /// kind — an allocation per cell, a per-glyph lookup — not to certify a
    /// frame budget.
    ///
    /// **The worst case is now a constant.** It used to be whatever grid the
    /// largest plausible window produced — 160×45 — and is now
    /// [`orbs_render::GRID`] itself, because no window makes it any bigger.
    #[test]
    fn rebuilding_the_worst_case_grid_is_not_a_frame_cost() {
        let grid = orbs_render::GRID;
        let mut frame = Frame::new(grid);
        let row = "X".repeat(usize::from(grid.cols));
        for y in 0..grid.rows {
            frame
                .painter(frame.area())
                .span(Pos::new(0, y), &Span::new(&row));
        }

        let mut mesh = empty_mesh();
        // Warm the allocations, as a running frame would have them.
        build(&frame, &MUTED_VIOLET, true, &mut mesh);
        assert_eq!(
            quads(&mesh),
            usize::from(grid.cols) * usize::from(grid.rows)
        );

        let rounds = 20;
        let start = std::time::Instant::now();
        for _ in 0..rounds {
            build(&frame, &MUTED_VIOLET, true, &mut mesh);
        }
        let each = start.elapsed() / rounds;

        println!(
            "worst-case grid rebuild: {each:?} for {} quads",
            quads(&mesh)
        );
        assert!(
            each < std::time::Duration::from_millis(16),
            "a worst-case rebuild took {each:?}, which is a whole frame at 60 Hz"
        );
    }
}
