use crate::{constants::get_version, event::Event};
use std::sync::mpsc::SyncSender;
use tray_item::TrayItem;

pub struct SystemTray(TrayItem);

impl SystemTray {
    pub fn init(tx: SyncSender<Event>) -> Result<Self, tray_item::TIError> {
        let mut tray = TrayItem::new(&format!("Nog - {}", get_version()), "logo.ico")?;
        {
            let tx = tx.clone();
            tray.add_menu_item("exit", move || {
                tx.send(Event::Exit).unwrap();
            })?;
        }
        Ok(Self(tray))
    }
}
