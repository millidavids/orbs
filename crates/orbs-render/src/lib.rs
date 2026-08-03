//! Frame / cell-buffer, layout, and semantic styling.
//!
//! `orbs-render` decides *what appears and where*. Frontends decide only *how a
//! cell is drawn*, and may add enrichment the others cannot reproduce (CRT
//! effects, audio, fidelity tiers) provided that enrichment carries no
//! information absent from the Frame.
//!
//! Built in Phase 0, item 2. See docs/ROADMAP.md.
