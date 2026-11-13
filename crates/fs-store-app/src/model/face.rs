//! Pattern: State (the face of the window: the saved choice, and the one asked for on the command line).

use fs_store_sources::Face;

use super::store::StoreModel;

impl StoreModel {
    /// The face to draw: the one given with `--variant`, else the saved choice (the storefront when nothing was saved).
    #[must_use]
    pub fn face(&self) -> Face {
        self.face_override.unwrap_or(self.settings.face)
    }

    /// Wears this face for this run only (`--variant`); nothing is saved.
    pub fn override_face(&mut self, face: Option<Face>) {
        self.face_override = face;
    }

    /// The person chose a face in the menu: it is worn at once and remembered (`settings.toml`). A choice in the window ends an
    /// override of the command line. A failure to save is told; the face still changes for this run.
    pub fn set_face(&mut self, face: Face) {
        self.problems.clear();
        self.face_override = None;
        self.settings.face = face;
        self.save_settings();
    }
}
