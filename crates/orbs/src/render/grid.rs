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
//! ~1.2 MB of allocation at a 160×45 grid, which defeated the reuse it was
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
pub(crate) fn build(frame: &Frame, theme: &Phosphor, scale: u16, showing: bool, mesh: &mut Mesh) {
    let mut geometry = Geometry::reclaim(mesh);

    let grid = frame.size();
    let cell_width = f32::from(CELL_WIDTH) * f32::from(scale);
    let cell_height = f32::from(CELL_HEIGHT) * f32::from(scale);
    let left = -(f32::from(grid.cols) * cell_width) / 2.0;
    let top = (f32::from(grid.rows) * cell_height) / 2.0;

    let position = |column: f32, row: f32| (left + column * cell_width, top - row * cell_height);

    for (row, cells) in frame.rows().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            if cell.is_blank() {
                continue;
            }
            let Some(index) = cp437::cp437_index(cell.glyph) else {
                continue;
            };
            let (x, y) = position(grid_offset(column), grid_offset(row));
            geometry.push_cell(
                x,
                y,
                cell_width,
                cell_height,
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
        let (x, y) = position(f32::from(caret.col), f32::from(caret.row));
        geometry.push_cell(
            x,
            y,
            cell_width,
            cell_height,
            theme.resolve(Style::BRIGHT).to_linear().to_f32_array(),
            atlas::uv(index, orbs_render::Presentation::Plain),
        );
    }

    geometry.commit(mesh);
}

/// An empty mesh with the attributes [`build`] fills.
pub(crate) fn empty_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
    mesh.insert_indices(Indices::U32(Vec::new()));
    mesh
}

/// `usize` grid coordinate to pixels, without a lossy cast lint at every site.
fn grid_offset(index: usize) -> f32 {
    u16::try_from(index).map_or(f32::from(u16::MAX), f32::from)
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

    fn built(frame: &Frame, scale: u16) -> Mesh {
        let mut mesh = empty_mesh();
        build(frame, &MUTED_VIOLET, scale, true, &mut mesh);
        mesh
    }

    #[test]
    fn one_quad_per_visible_cell() {
        assert_eq!(quads(&built(&frame_with("abc", 10, 1), 1)), 3);
    }

    #[test]
    fn blank_cells_cost_nothing() {
        // A space is the commonest glyph on screen; drawing a transparent quad
        // for every one of them would be most of the grid.
        assert_eq!(quads(&built(&Frame::new(GridSize::new(80, 22)), 1)), 0);
        assert_eq!(
            quads(&built(&frame_with("a b", 80, 22), 1)),
            2,
            "the space between should not be drawn"
        );
    }

    #[test]
    fn the_caret_is_drawn() {
        // `Frame` carries a cursor. A frontend that dropped it would show a
        // different screen from one that did not.
        let mut frame = frame_with("ab", 10, 1);
        let without = quads(&built(&frame, 1));

        frame.set_cursor(Some(Pos::new(3, 0)));
        assert_eq!(quads(&built(&frame, 1)), without + 1);
    }

    #[test]
    fn a_caret_outside_the_grid_draws_nothing() {
        let mut frame = frame_with("ab", 4, 1);
        frame.set_cursor(Some(Pos::new(99, 99)));
        // `Frame::set_cursor` refuses positions off the grid, so there is
        // nothing extra to draw.
        assert_eq!(quads(&built(&frame, 1)), 2);
    }

    #[test]
    fn the_grid_is_centred_on_the_origin() {
        let mesh = built(&frame_with("a", 1, 1), 1);
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
        let top = positions(&built(&frame, 1))[0][1];

        let mut frame = Frame::new(GridSize::new(2, 2));
        frame
            .painter(frame.area())
            .span(Pos::new(0, 1), &Span::new("a"));
        let below = positions(&built(&frame, 1))[0][1];

        assert!(top > below, "row 0 should sit above row 1");
    }

    #[test]
    fn scale_multiplies_the_cell_size_exactly() {
        // Integer scaling is what keeps a bitmap font crisp (§4).
        let single = positions(&built(&frame_with("a", 4, 1), 1));
        let triple = positions(&built(&frame_with("a", 4, 1), 3));

        let width = |p: &[[f32; 3]]| p[1][0] - p[0][0];
        assert!((width(&triple) - width(&single) * 3.0).abs() < 0.001);
    }

    #[test]
    fn every_quad_has_four_vertices_and_six_indices() {
        let mesh = built(&frame_with("hello", 10, 1), 2);
        assert_eq!(positions(&mesh).len(), quads(&mesh) * 4);
        assert_eq!(colours(&mesh).len(), positions(&mesh).len());
        assert_eq!(index_count(&mesh), quads(&mesh) * 6);
    }

    /// The bug the previous version shipped with.
    ///
    /// It kept a staging buffer, cloned it into the mesh every frame, and
    /// asserted only that the *staging* buffer kept its capacity — so ~1.2 MB of
    /// per-frame allocation passed a test named for reuse.
    #[test]
    fn rebuilding_allocates_nothing() {
        let mut mesh = empty_mesh();
        let frame = frame_with("something reasonably long", 40, 1);

        build(&frame, &MUTED_VIOLET, 1, true, &mut mesh);
        let capacity = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values.capacity(),
            _ => 0,
        };
        assert!(capacity > 0);

        for _ in 0..8 {
            build(&frame, &MUTED_VIOLET, 1, true, &mut mesh);
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
        assert_eq!(colours(&built(&frame, 1))[0], expected);
    }

    /// §4 warns that a full grid redrawing under a real-time siege is what the
    /// `bevy_text` path could not hold. This is the number that says whether the
    /// single-mesh path can.
    ///
    /// The bound is generous because tests run unoptimised; a release build is
    /// roughly an order of magnitude faster. It is here to catch a regression in
    /// kind — an allocation per cell, a per-glyph lookup — not to certify a
    /// frame budget.
    #[test]
    fn rebuilding_the_worst_case_grid_is_not_a_frame_cost() {
        let mut frame = Frame::new(GridSize::new(160, 45));
        let row = "X".repeat(160);
        for y in 0..45 {
            frame
                .painter(frame.area())
                .span(Pos::new(0, y), &Span::new(&row));
        }

        let mut mesh = empty_mesh();
        // Warm the allocations, as a running frame would have them.
        build(&frame, &MUTED_VIOLET, 1, true, &mut mesh);
        assert_eq!(quads(&mesh), 160 * 45);

        let rounds = 20;
        let start = std::time::Instant::now();
        for _ in 0..rounds {
            build(&frame, &MUTED_VIOLET, 1, true, &mut mesh);
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
