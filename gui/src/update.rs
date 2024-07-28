use std::sync::mpsc::{channel, Receiver, Sender};

use three_d::CpuMesh;

use crate::model::{Meshes, Settings};

/// A model update.
pub enum Update {
    /// An update for the whole model.
    New(Settings, Meshes),
    /// An update for the model settings.
    Settings(Settings),
    /// An update for the preview mesh.
    Preview(CpuMesh),
    /// An update for the model meshes.
    Meshes(Meshes),
}

/// An updater allowing to update a scene at runtime.
#[derive(Clone)]
pub struct Updater {
    sender: Sender<Update>,
}

impl Updater {
    /// Creates a new updater/receiver pair.
    #[must_use]
    pub fn new() -> (Self, Receiver<Update>) {
        let (sender, receiver) = channel();

        (Self { sender }, receiver)
    }

    /// Sends a update.
    pub fn send_update(&self, update: Update) {
        let _ = self.sender.send(update);
    }
}
