mod config;
mod desktop_app;
mod search;

use desktop_app::get_apps;
use gtk::gdk;
use gtk::{Application, ApplicationWindow, Box, Entry, Orientation};
use gtk::{ListBox, prelude::*};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn main() {
    let apps = get_apps();

    let app = Application::builder()
        .application_id("com.davidgl.rspot")
        .build();

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("rspot")
            .decorated(false)
            .default_width(600)
            .default_height(-1)
            .build();

        let container = Box::new(Orientation::Vertical, 0);
        let entry = Entry::new();
        entry.set_placeholder_text(Some("Buscar aplicaciones..."));

        let list = ListBox::new();
        list.set_visible(false);

        let css = gtk::CssProvider::new();
        css.load_from_data(
            "
                    entry {
                        background: rgba(40, 40, 40, 0.95);
                        border-radius: 12px;
                        padding: 8px 12px;
                        font-size: 14px;
                        color: white;
                        border: none;
                        box-shadow: none;
                    }
                    
                    listbox {
                        background: rgba(40, 40, 40, 0.95);
                        border-radius: 12px;
                        margin-top: 8px;
                        color: white;
                    }
                    
                    listbox row {
                        padding: 4px 12px;
                        border-radius: 8px;
                    }
                    
                    listbox row:selected {
                        background: rgba(82, 148, 226, 0.8);
                    }
                    window {
                        background: transparent;
                    }
                    .launcher-box {
                        background: transparent;
                        border-radius: 16px;
                    }

                    .launcher-content {
                        background: rgba(40, 40, 40, 0.95);
                        border-radius: 16px;
                        padding: 8px;
                    }

                    entry:focus {
                        box-shadow: none;
                        outline: none;
                    }

                    entry > text {
                        box-shadow: none;
                    }
                    
                    * {
                        outline: none;
                        -gtk-outline-radius: 0;
                    }
                ",
        );

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().unwrap(),
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let outer = Box::new(Orientation::Vertical, 0);
        outer.add_css_class("launcher-box");

        let inner = Box::new(Orientation::Vertical, 8);
        inner.add_css_class("launcher-content");

        inner.append(&entry);
        inner.append(&list);
        outer.append(&inner);

        window.set_child(Some(&outer));
        window.present();
        window.set_resizable(false);
        let list_clone = list.clone();
        let apps_clone = apps.clone();

        entry.connect_changed(move |e| {
            let query = e.text().to_string();

            while let Some(child) = list_clone.first_child() {
                list_clone.remove(&child);
            }

            if !query.is_empty() {
                let results = search::search_apps(&apps_clone, &query);
                for app in results.iter().take(10) {
                    let row_box = Box::new(Orientation::Horizontal, 8);

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
    });

    app.run();
}
