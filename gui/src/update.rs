use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
};

use config::Error;
use three_d::CpuMesh;
use winit::event_loop::EventLoopProxy;

use crate::model::{Meshes, Model};

/// An scene update event.
pub enum Update {
    New(Model, Meshes),
    Model(Model),
    Preview(CpuMesh),
    Meshes(Meshes),
    Error(Arc<Error>),
}

/// An updater allowing to update a scene at runtime.
#[derive(Clone)]
pub struct Updater {
    sender: Sender<Update>,
    event_loop_proxy: EventLoopProxy<()>,
}

impl Updater {
    /// Creates a new updater/receiver pair from an event loop proxy.
    #[must_use]
    pub fn from_event_loop_proxy(event_loop_proxy: EventLoopProxy<()>) -> (Self, Receiver<Update>) {
        let (sender, receiver) = channel();

        (
            Self {
                sender,
                event_loop_proxy,
            },
            receiver,
        )
    }

    /// Sends a update.
    pub fn send_update(&self, update: Update) {
        let _ = self.sender.send(update);
        let _ = self.event_loop_proxy.send_event(());
    }
}
