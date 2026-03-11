// dbus.rs
use std::sync::mpsc::Sender;
use zbus::interface;

pub struct RspotService {
    pub sender: Sender<()>,
}

#[interface(name = "com.davidgl.Rspot")]
impl RspotService {
    fn show(&self) {
        self.sender.send(()).ok();
    }
}
