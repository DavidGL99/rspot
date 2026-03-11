mod config;
mod dbus;
mod desktop_app;
mod search;
mod watcher;

use desktop_app::get_apps;
use gtk::EventControllerKey;
use gtk::gdk;
use gtk::{Application, ApplicationWindow, Box, Entry, Orientation};
use gtk::{ListBox, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::config::load_config;

fn main() {
    let apps = Arc::new(Mutex::new(get_apps()));
    let apps_for_watcher = apps.clone();

    let (reload_sender, reload_receiver) = std::sync::mpsc::channel::<()>();
    let reload_receiver = Arc::new(Mutex::new(reload_receiver));

    watcher::watch_apps(reload_sender);
    let config = load_config();

    let (sender, receiver) = mpsc::channel::<()>();
    let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));

    let app = Application::builder()
        .application_id("com.davidgl.rspot")
        .build();

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("rspot")
            .decorated(false)
            .default_width(config.window.width as i32)
            .default_height(config.window.max_height as i32)
            .build();

        let apps_clone_reload = apps.clone();
        let reload_receiver_clone = reload_receiver.clone();

        gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            if reload_receiver_clone.lock().unwrap().try_recv().is_ok() {
                *apps_clone_reload.lock().unwrap() = get_apps();
            }
            gtk::glib::ControlFlow::Continue
        });
        let container = Box::new(Orientation::Vertical, 0);
        let entry = Entry::new();
        entry.set_placeholder_text(Some("Buscar aplicaciones..."));

        let list = ListBox::new();
        list.set_visible(false);

        let css = gtk::CssProvider::new();
        let css_string = format!(
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
        }}

        .launcher-content listbox {{
            background: {bg};
            background-color: {bg};
            color: {font_color};
        }}

        .launcher-content listbox > row:not(:selected) {{
            background: {bg};
            background-color: {bg};
        }}

        .selected-row {{
            background-color: {accent};
            color: {font_color};
        }}

        entry:focus {{
            box-shadow: none;
            outline: none;
        }}

        entry > text {{
            box-shadow: none;
        }}

        * {{
            outline: none;
        }}
        .launcher-content listbox row:selected {{
            background-color: {accent};
            color: {font_color};
        }}

        listbox row:selected {{
            background-color: {accent};
            color: {font_color};
        }}

        .launcher-content listbox {{
            background: {bg};
            background-color: {bg};
            color: {font_color};
        }}

        .launcher-content listbox > row:not(:selected) {{
            background: {bg};
            background-color: {bg};
        }}
        listbox, listbox row {{
            background-color: {bg};
            color: {font_color};
        }}
    ",
            bg = config.colors.background,
            font_size = config.font.font_size,
            font_color = config.font.font_color,
            accent = config.colors.selected_item_color,
        );

        css.load_from_data(&css_string);

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().unwrap(),
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        list.add_css_class("navigation-sidebar");

        let outer = Box::new(Orientation::Vertical, 0);
        outer.add_css_class("launcher-box");
        outer.set_valign(gtk::Align::Start);
        outer.set_margin_top(150); // fijo desde arriba
        let inner = Box::new(Orientation::Vertical, 8);
        inner.add_css_class("launcher-content");

        inner.append(&entry);
        inner.append(&list);
        outer.append(&inner);

        window.set_child(Some(&outer));

        let window_clone3 = window.clone();
        let receiver_clone = receiver.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if receiver_clone.lock().unwrap().try_recv().is_ok() {
                window_clone3.show();
                window_clone3.present();
                window_clone3.grab_focus();
            }
            gtk::glib::ControlFlow::Continue
        });

        let key_controller = EventControllerKey::new();
        let list_clone2 = list.clone();
        let apps_clone2 = apps.lock().unwrap().clone();
        let entry_clone = entry.clone();
        let window_clone = window.clone();

        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);

        key_controller.connect_key_pressed(move |_, key, _, _| match key {
            gtk::gdk::Key::Escape => {
                window_clone.hide();
                entry_clone.set_text("");
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Return => {
                if let Some(row) = list_clone2.selected_row() {
                    if let Some(child) = row.child() {
                        let exec = child.widget_name().to_string();
                        launch_app(&exec);
                        window_clone.hide();
                        entry_clone.set_text("");
                    }
                }
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Down => {
                if list_clone2.is_visible() {
                    let current = list_clone2.selected_row().map(|r| r.index()).unwrap_or(-1);

                    // Quita clase de fila anterior
                    if let Some(prev) = list_clone2.row_at_index(current) {
                        prev.remove_css_class("selected-row");
                    }

                    let next = list_clone2.row_at_index(current + 1);
                    if let Some(ref row) = next {
                        row.add_css_class("selected-row");
                    }
                    list_clone2.select_row(next.as_ref());
                }
                gtk::glib::Propagation::Stop
            }

            gtk::gdk::Key::Up => {
                if list_clone2.is_visible() {
                    let current = list_clone2.selected_row().map(|r| r.index()).unwrap_or(1);

                    if current <= 0 {
                        // Quita clase y deselecciona
                        if let Some(row) = list_clone2.row_at_index(0) {
                            row.remove_css_class("selected-row");
                        }
                        list_clone2.unselect_all(); // ← deselecciona en el modelo
                        entry_clone.grab_focus();
                    } else {
                        if let Some(curr) = list_clone2.row_at_index(current) {
                            curr.remove_css_class("selected-row");
                        }
                        let prev = list_clone2.row_at_index(current - 1);
                        if let Some(ref row) = prev {
                            row.add_css_class("selected-row");
                        }
                        list_clone2.select_row(prev.as_ref());
                    }
                }
                gtk::glib::Propagation::Stop
            }
            _ => gtk::glib::Propagation::Proceed,
        });
        let entry_clone2 = entry.clone();
        let window_clone2 = window.clone();
        list.connect_row_activated(move |_, row| {
            if let Some(child) = row.child() {
                let exec = child.widget_name().to_string();
                launch_app(&exec);
                window_clone2.hide();
                entry_clone2.set_text("");
            }
        });
        window.add_controller(key_controller);
        let list_clone = list.clone();
        let apps_clone = apps.lock().unwrap().clone();

        entry.connect_changed(move |e| {
            let query = e.text().to_string();

            while let Some(child) = list_clone.first_child() {
                list_clone.remove(&child);
            }

            if !query.is_empty() {
                let results = search::search_apps(&apps_clone, &query);
                for app in results.iter().take(10) {
                    let row_box = Box::new(Orientation::Horizontal, 8);
                    row_box.set_widget_name(&app.exec); // ← guarda el exec

                    // Icono
                    if let Some(path) = &app.icon_path {
                        let image = gtk::Image::from_file(path);
                        image.set_pixel_size(32);
                        row_box.append(&image);
                    }

                    // Nombre
                    let label = gtk::Label::new(Some(&app.name));
                    label.set_halign(gtk::Align::Start);
                    row_box.append(&label);

                    list_clone.append(&row_box);
                    list_clone.set_visible(true);
                }
            } else {
                list_clone.set_visible(false);
            }
        });
        window.connect_close_request(move |w| {
            w.hide();
            gtk::glib::Propagation::Stop
        });
    });

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

    app.run();
}

fn launch_app(exec: &str) {
    let clean_exec: Vec<&str> = exec
        .split_whitespace()
        .filter(|a| !a.starts_with('%'))
        .collect();
    if !clean_exec.is_empty() {
        std::process::Command::new(clean_exec[0])
            .args(&clean_exec[1..])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok();
    }
}
