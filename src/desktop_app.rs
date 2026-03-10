use std::fs;
use std::path::PathBuf;
#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    pub exec: String,
    pub icon_path: Option<PathBuf>,
}

struct DesktopEntry {
    name: String,
    exec: String,
    icon: Option<String>,
}

fn parse_desktop_file(content: &str) -> Option<DesktopEntry> {
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut icon: Option<String> = None;

    let mut in_desktop_entry = false;

    for line in content.lines() {
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.contains('[') {
            continue;
        }

        match key {
            "Name" => name = Some(value.to_string()),
            "Exec" => exec = Some(value.to_string()),
            "Icon" => icon = Some(value.to_string()),
            "Type" => {
                if value != "Application" {
                    return None;
                }
            }
            "NoDisplay" => {
                if value == "true" {
                    return None;
                }
            }
            _ => {}
        }
    }

    let (name, exec) = name.zip(exec)?;
    Some(DesktopEntry { name, exec, icon })
}

fn resolve_icon(name: &str) -> Option<PathBuf> {
    let paths_to_search = vec![
        PathBuf::from("/usr/share/icons/Papirus/48x48/apps/").join(format!("{}.svg", name)),
        PathBuf::from("/usr/share/icons/Papirus/48x48/apps/").join(format!("{}.png", name)),
        PathBuf::from("/usr/share/icons/hicolor/48x48/apps/").join(format!("{}.svg", name)),
        PathBuf::from("/usr/share/icons/hicolor/48x48/apps").join(format!("{}.png", name)),
        PathBuf::from("/usr/share/pixmaps/").join(format!("{}.svg", name)),
        PathBuf::from("/usr/share/pixmaps/").join(format!("{}.png", name)),
        PathBuf::from("/usr/share/icons/hicolor/scalable/apps/").join(format!("{}.svg", name)),
        PathBuf::from("/usr/share/icons/hicolor/64x64/apps/").join(format!("{}.svg", name)),
        PathBuf::from("/usr/share/icons/hicolor/64x64/apps/").join(format!("{}.png", name)),
        PathBuf::from("/usr/share/icons/Papirus/48x48/devices/").join(format!("{}.svg", name)),
        PathBuf::from("/usr/share/icons/hicolor/48x48/devices/").join(format!("{}.svg", name)),
    ];
    for path in paths_to_search {
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub fn get_apps() -> Vec<App> {
    let mut apps: Vec<App> = Vec::new();

    let home = std::env::var("HOME").unwrap_or_default();
    let flatpak_dir = format!("{}/.local/share/flatpak/exports/share/applications", home);
    let directories = vec!["/usr/share/applications".to_string(), flatpak_dir];
    for directory in directories {
        let entries = match fs::read_dir(&directory) {
            Ok(e) => e,
            Err(_) => continue,
        };
        apps.extend(
            entries
                .filter(|x| {
                    x.as_ref()
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .ends_with(".desktop")
                })
                .filter_map(|entry| {
                    let content = fs::read_to_string(entry.ok()?.path()).ok()?;
                    let app = parse_desktop_file(&content)?;
                    let icon_path = app.icon.as_deref().and_then(resolve_icon);
                    Some(App {
                        name: app.name,
                        exec: app.exec,
                        icon_path,
                    })
                }),
        );
    }
    apps
}
