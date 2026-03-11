use notify::{Event, RecursiveMode, Watcher, recommended_watcher};
use std::path::Path;
use std::sync::mpsc::Sender;

pub fn watch_apps(sender: Sender<()>) {
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();

        let mut watcher = recommended_watcher(tx).unwrap();

        watcher
            .watch(
                Path::new("/usr/share/applications"),
                RecursiveMode::NonRecursive,
            )
            .unwrap();

        // También observa flatpak
        let home = std::env::var("HOME").unwrap_or_default();
        let flatpak_path = format!("{}/.local/share/flatpak/exports/share/applications", home);
        let flatpak_path = Path::new(&flatpak_path);
        if flatpak_path.exists() {
            watcher
                .watch(flatpak_path, RecursiveMode::NonRecursive)
                .unwrap();
        }

        for event in rx {
            if event.is_ok() {
                sender.send(()).ok();
            }
        }
    });
}
