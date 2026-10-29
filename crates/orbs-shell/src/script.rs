//! A `;` script and an editor's keystrokes, driven through the sim as a
//! player's would be. One copy for the dump and the tour.

use orbs_sim::{Augur, Scrivener, Sim};

use super::install::Install;
use super::{Editor, EditorMode, EditorOutcome};

/// Commands are separated by this, so one shell word can drive a session.
pub(crate) const SEPARATOR: char = ';';

/// Submit every `;`-separated command in `script`, stepping between them.
///
/// While `installing`, a line goes through the installation, as in both
/// frontends.
pub(crate) fn drive(
    sim: &mut Sim,
    installing: &mut Option<Install>,
    script: &str,
    augury: Option<&dyn Augur>,
) {
    for line in script
        .split(SEPARATOR)
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if installing.is_some() {
            crate::install::drive_dump(sim, installing, line, augury);
            continue;
        }
        sim.submit_with(line, augury);
        sim.step();
    }
}

/// The editor a pending `scribe` asked for, with nothing typed into it.
///
/// `editing::open_requested`'s two lines.
pub(crate) fn open(sim: &mut Sim) -> Option<Editor> {
    let request = sim.opening()?;
    Some(Editor::open(&request.name, &request.domain, &request.lines))
}

/// `editor` with `script` typed into it, or [`None`] once the script saved and
/// closed it.
///
/// Through the [`Editor`] methods the key handler calls, every character
/// through `insert`, so dump and game agree. The editor's state says what a
/// segment is. A save writes for real, through `scrivener` if there is one.
pub(crate) fn type_into(
    sim: &mut Sim,
    mut editor: Editor,
    script: &str,
    scrivener: Option<&dyn Scrivener>,
) -> Option<Editor> {
    // `\n` as two characters, because a shell word carries it that way.
    let mut wrote_a_line = false;
    for segment in script.replace("\\n", "\n").split('\n') {
        // The one token a keyboard has and a shell word does not.
        if segment.trim() == "<esc>" {
            editor.escape();
            continue;
        }

        // In the buffer, a break goes **between** lines rather than after each
        // one: a trailing newline would put an empty line at the end of every
        // spell, and the log would report one line more than was typed.
        if editor.mode() == EditorMode::Editing && wrote_a_line {
            editor.enter();
        }
        for character in segment.chars() {
            editor.type_text(&character.to_string());
        }

        if editor.mode() == EditorMode::Editing {
            wrote_a_line = true;
            continue;
        }
        // Command state: `Enter` runs the word.
        match editor.enter() {
            Some(EditorOutcome::Save) => {
                wrote(sim, &editor, scrivener);
                editor.saved();
            }
            Some(EditorOutcome::SaveAndClose) => {
                wrote(sim, &editor, scrivener);
                sim.step();
                return None;
            }
            None => {}
        }
    }
    Some(editor)
}

/// Save the buffer, so the spell never depends on how it closed.
fn wrote(sim: &mut Sim, editor: &Editor, scrivener: Option<&dyn Scrivener>) {
    match scrivener {
        Some(reader) => sim.write_spell_reading(editor.name(), editor.lines(), reader),
        None => sim.write_spell(editor.name(), editor.lines()),
    }
}
