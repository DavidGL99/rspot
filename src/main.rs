mod config;
mod dbus;
mod desktop_app;
mod search;
mod watcher;

use std::sync::{Arc, Mutex, mpsc};

use gtk::{
    Application, ApplicationWindow, Box, Entry, EventControllerKey, ListBox, Orientation,
    prelude::*,
};

use config::load_config;
use desktop_app::get_apps;

fn main() {
    let apps = Arc::new(Mutex::new(get_apps()));
    let config = load_config();

    // Channel for D-Bus show events
    let (dbus_sender, dbus_receiver) = mpsc::channel::<()>();
    let dbus_receiver = Arc::new(Mutex::new(dbus_receiver));

    // Channel for app reload events from filesystem watcher
    let (reload_sender, reload_receiver) = mpsc::channel::<()>();
    let reload_receiver = Arc::new(Mutex::new(reload_receiver));

    watcher::watch_apps(reload_sender);
    spawn_dbus_server(dbus_sender);

    let app = Application::builder()
        .application_id("com.davidgl.rspot")
        .build();

    app.connect_activate(move |app| {
        let window = build_window(app, &config);
        let entry = Entry::new();
        let list = ListBox::new();

        entry.set_placeholder_text(Some("Search applications..."));
        list.set_visible(false);
        list.add_css_class("navigation-sidebar");

        apply_css(&config);
        build_layout(&window, &entry, &list);
        setup_reload_watcher(&apps, &reload_receiver);
        setup_dbus_listener(&window, &dbus_receiver);
        setup_key_controller(&window, &entry, &list);
        setup_row_activated(&window, &entry, &list);
        setup_search(&entry, &list, &apps);

        window.connect_close_request(|w| {
            w.hide();
            gtk::glib::Propagation::Stop
        });

        // Show briefly so GTK calculates layout, then hide
        window.present();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(5), {
            let w = window.clone();
            move || w.hide()
        });
    });

    app.run();
}

fn build_window(app: &Application, config: &config::Config) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(app)
        .title("rspot")
        .decorated(false)
        .default_width(config.window.width as i32)
        .default_height(config.window.max_height as i32)
        .build()
}

fn build_layout(window: &ApplicationWindow, entry: &Entry, list: &ListBox) {
    let outer = Box::new(Orientation::Vertical, 0);
    outer.add_css_class("launcher-box");
    outer.set_valign(gtk::Align::Start);
    outer.set_margin_top(150);

    let inner = Box::new(Orientation::Vertical, 8);
    inner.add_css_class("launcher-content");
    inner.append(entry);
    inner.append(list);

    outer.append(&inner);
    window.set_child(Some(&outer));
}

fn apply_css(config: &config::Config) {
    let css = gtk::CssProvider::new();
    let bg = &config.colors.background;
    let font_color = &config.font.font_color;
    let font_size = config.font.font_size;
    let accent = &config.colors.selected_item_color;
    let opacity = config.colors.opacity;

    css.load_from_data(&format!(
        "
        entry {{
            background: {bg};
            border-radius: 12px;
            padding: 8px 12px;
            font-size: {font_size}px;
            color: {font_color};
            border: none;
            box-shadow: none;
        }}
        entry:focus, entry > text {{
            box-shadow: none;
            outline: none;
        }}
        listbox {{
            background: {bg};
            border-radius: 12px;
            margin-top: 8px;
            color: {font_color};
        }}
        listbox row {{
            padding: 4px 12px;
            border-radius: 8px;
        }}
        window {{
            background: transparent;
        }}
        .launcher-box {{
            background: transparent;
            border-radius: 16px;
        }}
        .launcher-content {{
            background: {bg};
            border-radius: 16px;
            padding: 8px;
            opacity: {opacity};

        }}
        .launcher-content listbox,
        .launcher-content listbox > row:not(:selected) {{
            background: {bg};
            background-color: {bg};
            color: {font_color};
        }}
        .selected-row {{
            background-color: {accent};
            color: {font_color};
        }}
        * {{ outline: none; }}
    "
    ));

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn setup_reload_watcher(
    apps: &Arc<Mutex<Vec<desktop_app::App>>>,
    reload_receiver: &Arc<Mutex<mpsc::Receiver<()>>>,
) {
    let apps = apps.clone();
    let reload_receiver = reload_receiver.clone();

    gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        if reload_receiver.lock().unwrap().try_recv().is_ok() {
            *apps.lock().unwrap() = get_apps();
        }
        gtk::glib::ControlFlow::Continue
    });
}

fn setup_dbus_listener(window: &ApplicationWindow, dbus_receiver: &Arc<Mutex<mpsc::Receiver<()>>>) {
    let window = window.clone();
    let dbus_receiver = dbus_receiver.clone();

    gtk::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if dbus_receiver.lock().unwrap().try_recv().is_ok() {
            window.show();
            window.present();
            window.grab_focus();
        }
        gtk::glib::ControlFlow::Continue
    });
}

fn setup_key_controller(window: &ApplicationWindow, entry: &Entry, list: &ListBox) {
    let key_controller = EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);

    let entry = entry.clone();
    let list = list.clone();
    let window_cln = window.clone();

    key_controller.connect_key_pressed(move |_, key, _, _| match key {
        gtk::gdk::Key::Escape => {
            window_cln.hide();
            entry.set_text("");
            gtk::glib::Propagation::Stop
        }
        gtk::gdk::Key::Return => {
            if let Some(row) = list.selected_row() {
                if let Some(child) = row.child() {
                    launch_app(&child.widget_name());
                    window_cln.hide();
                    entry.set_text("");
                }
            }
            gtk::glib::Propagation::Stop
        }
        gtk::gdk::Key::Down => {
            if list.is_visible() {
                let current = list.selected_row().map(|r| r.index()).unwrap_or(-1);
                if let Some(row) = list.row_at_index(current) {
                    row.remove_css_class("selected-row");
                }
                if let Some(next) = list.row_at_index(current + 1) {
                    next.add_css_class("selected-row");
                    list.select_row(Some(&next));
                }
            }
            gtk::glib::Propagation::Stop
        }
        gtk::gdk::Key::Up => {
            if list.is_visible() {
                let current = list.selected_row().map(|r| r.index()).unwrap_or(1);
                if current <= 0 {
                    if let Some(row) = list.row_at_index(0) {
                        row.remove_css_class("selected-row");
                    }
                    list.unselect_all();
                    entry.grab_focus();
                } else {
                    if let Some(curr) = list.row_at_index(current) {
                        curr.remove_css_class("selected-row");
                    }
                    if let Some(prev) = list.row_at_index(current - 1) {
                        prev.add_css_class("selected-row");
                        list.select_row(Some(&prev));
                    }
                }
            }
            gtk::glib::Propagation::Stop
        }
        _ => gtk::glib::Propagation::Proceed,
    });

    window.add_controller(key_controller);
}

fn setup_row_activated(window: &ApplicationWindow, entry: &Entry, list: &ListBox) {
    let window = window.clone();
    let entry = entry.clone();

    list.connect_row_activated(move |_, row| {
        if let Some(child) = row.child() {
            launch_app(&child.widget_name());
            window.hide();
            entry.set_text("");
        }
    });
}

fn setup_search(entry: &Entry, list: &ListBox, apps: &Arc<Mutex<Vec<desktop_app::App>>>) {
    let list = list.clone();
    let apps = apps.lock().unwrap().clone();

    entry.connect_changed(move |e| {
        let query = e.text().to_string();

        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        if query.is_empty() {
            list.set_visible(false);
            return;
        }

        let results = search::search_apps(&apps, &query);
        for app in results.iter().take(10) {
            let row = Box::new(Orientation::Horizontal, 8);
            row.set_widget_name(&app.exec);

            if let Some(path) = &app.icon_path {
                let image = gtk::Image::from_file(path);
                image.set_pixel_size(32);
                row.append(&image);
            }

            let label = gtk::Label::new(Some(&app.name));
            label.set_halign(gtk::Align::Start);
            row.append(&label);

            list.append(&row);
        }

        list.set_visible(true);
    });
}

fn spawn_dbus_server(sender: mpsc::Sender<()>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = dbus::RspotService { sender };
            let _conn = zbus::ConnectionBuilder::session()
                .unwrap()
                .name("com.davidgl.Rspot")
                .unwrap()
                .serve_at("/com/davidgl/Rspot", service)
                .unwrap()
                .build()
                .await
                .unwrap();

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        });
    });
}

fn launch_app(exec: &str) {
    let args: Vec<&str> = exec
        .split_whitespace()
        .filter(|a| !a.starts_with('%'))
        .collect();

    if let Some((cmd, rest)) = args.split_first() {
        std::process::Command::new(cmd)
            .args(rest)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok();
    }
}
