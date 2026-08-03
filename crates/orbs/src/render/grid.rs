//! The cell grid, as one mesh.
//!
//! DESIGN.md §4 rules out the `bevy_text`/`bevy_ui` path outright — it "will not
//! hold a full grid redrawing under a real-time siege" — and names the
//! alternative: *"single mesh with glyph atlas and instanced quads, or a custom
//! material driven by a cell-index texture."*
//!
//! This is the first of those. Every visible cell contributes four vertices to
//! one mesh, so the whole screen is **one draw call** regardless of grid size,
//! and the buffers are rebuilt in place each frame rather than reallocated.
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
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use orbs_render::{CELL_HEIGHT, CELL_WIDTH, Frame, cp437};

use super::atlas;
use super::palette::Phosphor;

/// Vertex buffers for the grid, reused across frames.
#[derive(Debug, Default)]
pub(crate) struct GridBuffers {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colours: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl GridBuffers {
    /// Discard the previous frame's geometry, keeping the allocations.
    fn clear(&mut self) {
        self.positions.clear();
        self.uvs.clear();
        self.colours.clear();
        self.indices.clear();
    }

    /// How many quads the last build emitted.
    #[cfg(test)]
    pub(crate) fn quads(&self) -> usize {
        self.positions.len() / 4
    }
}

/// Rebuild `mesh` from `frame`.
///
/// Blank cells contribute nothing: a space is the commonest glyph on screen by a
/// wide margin, and a quad that samples a fully transparent texel is pure cost.
pub(crate) fn build(
    frame: &Frame,
    theme: &Phosphor,
    scale: u16,
    buffers: &mut GridBuffers,
    mesh: &mut Mesh,
) {
    buffers.clear();

    let grid = frame.size();
    let cell_width = f32::from(CELL_WIDTH) * f32::from(scale);
    let cell_height = f32::from(CELL_HEIGHT) * f32::from(scale);
    let left = -(f32::from(grid.cols) * cell_width) / 2.0;
    let top = (f32::from(grid.rows) * cell_height) / 2.0;

    for (row, cells) in frame.rows().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            if cell.is_blank() {
                continue;
            }
            let Some(index) = cp437::cp437_index(cell.glyph) else {
                continue;
            };

            let x = left + row_offset(column) * cell_width;
            let y = top - row_offset(row) * cell_height;
            let colour = theme.resolve(cell.style).to_linear().to_f32_array();
            let (u0, v0, u1, v1) = atlas::uv(index, cell.style.presentation);

            let base = u32::try_from(buffers.positions.len()).unwrap_or(0);
            // Top-left, top-right, bottom-right, bottom-left.
            buffers.positions.extend_from_slice(&[
                [x, y, 0.0],
                [x + cell_width, y, 0.0],
                [x + cell_width, y - cell_height, 0.0],
                [x, y - cell_height, 0.0],
            ]);
            buffers
                .uvs
                .extend_from_slice(&[[u0, v0], [u1, v0], [u1, v1], [u0, v1]]);
            buffers.colours.extend_from_slice(&[colour; 4]);
            buffers.indices.extend_from_slice(&[
                base,
                base + 2,
                base + 1,
                base,
                base + 3,
                base + 2,
            ]);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, buffers.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, buffers.uvs.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, buffers.colours.clone());
    mesh.insert_indices(Indices::U32(buffers.indices.clone()));
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
fn row_offset(index: usize) -> f32 {
    u16::try_from(index).map_or(f32::from(u16::MAX), f32::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::{GridSize, Pos, Span, Style};

    use crate::render::palette::MUTED_VIOLET;

    fn frame_with(text: &str, cols: u16, rows: u16) -> Frame {
        let mut frame = Frame::new(GridSize::new(cols, rows));
        frame
            .painter(frame.area())
            .span(Pos::ORIGIN, &Span::new(text));
        frame
    }

    fn built(frame: &Frame, scale: u16) -> (GridBuffers, Mesh) {
        let mut buffers = GridBuffers::default();
        let mut mesh = empty_mesh();
        build(frame, &MUTED_VIOLET, scale, &mut buffers, &mut mesh);
        (buffers, mesh)
    }

    #[test]
    fn one_quad_per_visible_cell() {
        let (buffers, _) = built(&frame_with("abc", 10, 1), 1);
        assert_eq!(buffers.quads(), 3);
    }

    #[test]
    fn blank_cells_cost_nothing() {
        // A space is the commonest glyph on screen; drawing a transparent quad
        // for every one of them would be most of the grid.
        let empty = Frame::new(GridSize::new(80, 22));
        let (buffers, _) = built(&empty, 1);
        assert_eq!(buffers.quads(), 0);

        let (sparse, _) = built(&frame_with("a b", 80, 22), 1);
        assert_eq!(sparse.quads(), 2, "the space between should not be drawn");
    }

    #[test]
    fn the_grid_is_centred_on_the_origin() {
        // One cell at scale 1: it should straddle the origin.
        let (buffers, _) = built(&frame_with("a", 1, 1), 1);
        let xs: Vec<f32> = buffers.positions.iter().map(|p| p[0]).collect();
        let ys: Vec<f32> = buffers.positions.iter().map(|p| p[1]).collect();

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
        let (top_left, _) = built(&frame, 1);

        let mut frame = Frame::new(GridSize::new(2, 2));
        frame
            .painter(frame.area())
            .span(Pos::new(0, 1), &Span::new("a"));
        let (below, _) = built(&frame, 1);

        assert!(
            top_left.positions[0][1] > below.positions[0][1],
            "row 0 should sit above row 1"
        );
    }

    #[test]
    fn scale_multiplies_the_cell_size_exactly() {
        // Integer scaling is what keeps a bitmap font crisp (§4).
        let (single, _) = built(&frame_with("a", 4, 1), 1);
        let (triple, _) = built(&frame_with("a", 4, 1), 3);

        let width = |b: &GridBuffers| b.positions[1][0] - b.positions[0][0];
        assert!((width(&triple) - width(&single) * 3.0).abs() < 0.001);
    }

    #[test]
    fn every_quad_has_four_vertices_and_six_indices() {
        let (buffers, _) = built(&frame_with("hello", 10, 1), 2);
        assert_eq!(buffers.positions.len(), buffers.quads() * 4);
        assert_eq!(buffers.uvs.len(), buffers.positions.len());
        assert_eq!(buffers.colours.len(), buffers.positions.len());
        assert_eq!(buffers.indices.len(), buffers.quads() * 6);
    }

    #[test]
    fn rebuilding_reuses_the_allocation() {
        let mut buffers = GridBuffers::default();
        let mut mesh = empty_mesh();
        let frame = frame_with("something reasonably long", 40, 1);

        build(&frame, &MUTED_VIOLET, 1, &mut buffers, &mut mesh);
        let capacity = buffers.positions.capacity();
        build(&frame, &MUTED_VIOLET, 1, &mut buffers, &mut mesh);

        assert_eq!(buffers.positions.capacity(), capacity);
    }

    #[test]
    fn style_reaches_the_vertex_colours() {
        let mut frame = Frame::new(GridSize::new(4, 1));
        frame
            .painter(frame.area())
            .span(Pos::ORIGIN, &Span::new("x").with_style(Style::DANGER));
        let (buffers, _) = built(&frame, 1);

        let expected = MUTED_VIOLET
            .resolve(Style::DANGER)
            .to_linear()
            .to_f32_array();
        assert_eq!(buffers.colours[0], expected);
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

        let mut buffers = GridBuffers::default();
        let mut mesh = empty_mesh();
        // Warm the allocations, as a running frame would have them.
        build(&frame, &MUTED_VIOLET, 1, &mut buffers, &mut mesh);

        let rounds = 20;
        let start = std::time::Instant::now();
        for _ in 0..rounds {
            build(&frame, &MUTED_VIOLET, 1, &mut buffers, &mut mesh);
        }
        let each = start.elapsed() / rounds;

        println!("worst-case grid rebuild: {each:?} for 7200 quads");
        assert!(
            each < std::time::Duration::from_millis(16),
            "a worst-case rebuild took {each:?}, which is a whole frame at 60 Hz"
        );
    }

    #[test]
    fn a_full_worst_case_grid_builds() {
        // 160x45 is the largest grid §9's tier table produces.
        let mut frame = Frame::new(GridSize::new(160, 45));
        let row = "X".repeat(160);
        for y in 0..45 {
            frame
                .painter(frame.area())
                .span(Pos::new(0, y), &Span::new(&row));
        }
        let (buffers, _) = built(&frame, 1);
        assert_eq!(buffers.quads(), 160 * 45);
    }
}
