use app::{App, get_apps};
use config::{Config, load_config};
use iced::keyboard::key::Named;
use iced::widget::{Space, column, container, image, row, scrollable, svg, text, text_input};
use iced::{Application, Element, Settings, Task, keyboard, window};
use search::search_apps;
use std::ffi::OsStr;
mod app;
mod config;
mod search;

#[derive(Debug, Clone)]
enum Message {
    QueryChanged(String),
    AppLaunched,
    Dismissed,
    SelectNext,
    SelectPrev,
    WindowOpened(window::Id),
}

struct Rspot {
    query: String,
    apps: Vec<App>,
    config: Config,
    selected: Option<usize>,
    window_id: Option<window::Id>,
}
impl Default for Rspot {
    fn default() -> Self {
        Self {
            query: String::new(),
            apps: get_apps(),
            config: load_config(),
            selected: None,
            window_id: None,
        }
    }
}

impl Rspot {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::QueryChanged(query) => {
                self.query = query;
                self.selected = Some(0);
            }
            Message::AppLaunched => {
                if let Some(index) = self.selected {
                    let results = search_apps(&self.apps, &self.query);
                    if let Some(app) = results.get(index) {
                        let exec = &app.exec;
                        let clean_exec = exec
                            .split_whitespace()
                            .filter(|arg| !arg.starts_with('%'))
                            .collect::<Vec<_>>();

                        std::process::Command::new(clean_exec[0])
                            .args(&clean_exec[1..])
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .spawn()
                            .ok();
                    }
                }
            }
            Message::Dismissed => {
                std::process::exit(0);
            }
            Message::SelectNext => {
                println!("SelectNext, selected: {:?}", self.selected);

                let results = search_apps(&self.apps, &self.query);
                if let Some(i) = self.selected {
                    self.selected = Some((i + 1).min(results.len() - 1));
                }
            }
            Message::SelectPrev => {
                if let Some(i) = self.selected {
                    self.selected = Some(i.saturating_sub(1));
                }
            }
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let bg_color: iced::Color = hex_to_color(&self.config.colors.background);

        let results = if self.query.is_empty() {
            vec![]
        } else {
            let mut r = search_apps(&self.apps, &self.query);
            r.truncate(10);
            r
        };
        let input = text_input("Buscar aplicaciones...", &self.query)
            .on_input(Message::QueryChanged)
            .on_submit(Message::AppLaunched)
            .style(move |_, status| text_input::Style {
                background: bg_color.into(),
                border: iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                placeholder: iced::Color::from_rgb(0.5, 0.5, 0.5),
                value: iced::Color::WHITE,
                selection: iced::Color::from_rgb(0.32, 0.58, 0.89),
                icon: iced::Color::WHITE,
            });
        let list = results
            .iter()
            .enumerate()
            .map(|(i, app)| {
                let is_selected = self.selected == Some(i);

                let icon: iced::Element<Message> = match &app.icon_path {
                    Some(path) => match path.extension().and_then(OsStr::to_str) {
                        Some("svg") => svg(svg::Handle::from_path(path))
                            .width(32)
                            .height(32)
                            .content_fit(iced::ContentFit::Contain)
                            .into(),
                        Some("png") => image(path)
                            .width(32)
                            .height(32)
                            .content_fit(iced::ContentFit::Contain)
                            .into(),
                        _ => text("").into(),
                    },
                    None => text("").into(),
                };
                let name = text(&app.name);

                let item = row![icon, name]
                    .spacing(10)
                    .padding(8)
                    .align_y(iced::Alignment::Center);
                if is_selected {
                    container(item)
                        .style(|_| container::Style {
                            background: Some(iced::Color::from_rgb(0.32, 0.58, 0.89).into()),
                            ..Default::default()
                        })
                        .width(iced::Length::Fill)
                        .into()
                } else {
                    container(item).width(iced::Length::Fill).into()
                }
            })
            .collect::<Vec<_>>();
        let content = if list.is_empty() {
            column![input]
        } else {
            column![
                input,
                scrollable(column(list)).style(move |_, _| scrollable::Style {
                    container: container::Style {
                        background: Some(bg_color.into()),
                        ..Default::default()
                    },
                    vertical_rail: scrollable::Rail {
                        background: None,
                        border: iced::Border::default(),
                        scroller: scrollable::Scroller {
                            color: iced::Color::TRANSPARENT,
                            border: iced::Border::default(),
                        },
                    },
                    horizontal_rail: scrollable::Rail {
                        background: None,
                        border: iced::Border::default(),
                        scroller: scrollable::Scroller {
                            color: iced::Color::TRANSPARENT,
                            border: iced::Border::default(),
                        },
                    },
                    gap: None,
                })
            ]
        };

        container(content)
            .style(move |_| container::Style {
                background: Some(bg_color.into()),
                ..Default::default()
            })
            .width(iced::Length::Fill)
            .height(iced::Length::Shrink)
            .into()
    }
}

fn hex_to_color(hex: &str) -> iced::Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    iced::Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
fn main() -> iced::Result {
    iced::application("rspot", Rspot::update, Rspot::view)
        .decorations(false)
        .window_size(iced::Size::new(600.0, 40.0))
        .style(|_, _| iced::application::Appearance {
            background_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::WHITE,
        })
        .subscription(|_| {
            iced::Subscription::batch([keyboard::on_key_press(|key, _modifiers| match key {
                keyboard::Key::Named(Named::ArrowDown) => Some(Message::SelectNext),
                keyboard::Key::Named(Named::ArrowUp) => Some(Message::SelectPrev),
                keyboard::Key::Named(Named::Escape) => Some(Message::Dismissed),
                _ => None,
            })])
        })
        .subscription(|_| {
            iced::Subscription::batch([
                iced::window::close_events().map(|_| Message::Dismissed),
                iced::event::listen_with(|event, _, id| match event {
                    iced::Event::Window(window::Event::Opened { .. }) => {
                        Some(Message::WindowOpened(id))
                    }
                    _ => None,
                }),
            ])
        })
        .run()
}
