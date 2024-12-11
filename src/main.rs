use iced::{
    widget::{button, column, container, row, text, image as iced_image},
    Application, Command, Element, Settings, Theme, Length,
    executor, window, keyboard, event, subscription,
};
use std::{path::PathBuf, collections::HashMap};
use walkdir::WalkDir;
use image::open as image_open;

struct PhotoSelector {
    photo_paths: Vec<PathBuf>,
    cached_photos: HashMap<usize, Photo>,
    link_raw_jpg: bool,
    selected_photos: HashMap<PathBuf, bool>,
    current_photo_index: usize,
}

impl PhotoSelector {
    fn cleanup_cache(&mut self) {
        let keep_indices: Vec<usize> = vec![
            self.current_photo_index,
            self.current_photo_index.saturating_add(1)
        ];
        
        self.cached_photos.retain(|&index, _| keep_indices.contains(&index));
    }
}

#[derive(Debug, Clone)]
struct Photo {
    path: PathBuf,
    exif_data: String,
    has_raw: bool,
    handle: iced::widget::image::Handle,
}

#[derive(Debug, Clone)]
enum Message {
    LoadPhotoPaths(Vec<PathBuf>),
    PhotoLoaded((usize, Photo)),
    ToggleLinkRawJpg(bool),
    ToggleSelect(PathBuf),
    DeleteSelected,
    NextPhoto,
    PreviousPhoto,
    SelectPhoto,
    DeletePhoto(usize),
    KeyPressed(keyboard::Event),
}

impl Application for PhotoSelector {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            PhotoSelector {
                photo_paths: Vec::new(),
                cached_photos: HashMap::new(),
                link_raw_jpg: true,
                selected_photos: HashMap::new(),
                current_photo_index: 0,
            },
            Command::perform(load_photo_paths(), Message::LoadPhotoPaths),
        )
    }

    fn title(&self) -> String {
        String::from("Photo Selector")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        subscription::events_with(|event, _| {
            if let event::Event::Keyboard(key_event) = event {
                Some(Message::KeyPressed(key_event))
            } else {
                None
            }
        })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadPhotoPaths(paths) => {
                self.photo_paths = paths;
                if !self.photo_paths.is_empty() {
                    let load_commands: Vec<Command<Message>> = self.photo_paths.iter()
                        .take(2)
                        .enumerate()
                        .map(|(i, path)| {
                            Command::perform(
                                load_single_photo(path.clone(), i),
                                Message::PhotoLoaded
                            )
                        })
                        .collect();
                    Command::batch(load_commands)
                } else {
                    Command::none()
                }
            }
            Message::PhotoLoaded((index, photo)) => {
                self.cached_photos.insert(index, photo);
                while self.cached_photos.len() > 2 {
                    let oldest = self.cached_photos
                        .keys()
                        .min()
                        .copied()
                        .unwrap_or(0);
                    self.cached_photos.remove(&oldest);
                }
                Command::none()
            }
            Message::ToggleLinkRawJpg(value) => {
                self.link_raw_jpg = value;
                Command::none()
            }
            Message::ToggleSelect(path) => {
                let selected = self.selected_photos.entry(path).or_insert(false);
                *selected = !*selected;
                Command::none()
            }
            Message::DeleteSelected => {
                let paths_to_remove: Vec<PathBuf> = self.photo_paths.iter()
                    .filter(|path| *self.selected_photos.get(*path).unwrap_or(&false))
                    .cloned()
                    .collect();

                for path in paths_to_remove {
                    if self.link_raw_jpg && check_raw_exists(&path) {
                        let raw_path = get_raw_path(&path);
                        if let Err(e) = std::fs::remove_file(&raw_path) {
                            eprintln!("Failed to delete RAW file: {}", e);
                        }
                    }
                    if let Err(e) = std::fs::remove_file(&path) {
                        eprintln!("Failed to delete JPG file: {}", e);
                    }
                    if let Some(index) = self.photo_paths.iter().position(|p| p == &path) {
                        self.photo_paths.remove(index);
                        self.cached_photos.remove(&index);
                    }
                }
                self.selected_photos.clear();
                Command::none()
            }
            Message::NextPhoto => {
                if self.current_photo_index < self.photo_paths.len() - 1 {
                    self.current_photo_index += 1;
                    self.cleanup_cache();
                    if self.current_photo_index + 1 < self.photo_paths.len() {
                        Command::perform(
                            load_single_photo(
                                self.photo_paths[self.current_photo_index + 1].clone(),
                                self.current_photo_index + 1
                            ),
                            Message::PhotoLoaded
                        )
                    } else {
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            Message::PreviousPhoto => {
                if self.current_photo_index > 0 {
                    self.current_photo_index -= 1;
                    self.cleanup_cache();
                    Command::perform(
                        load_single_photo(
                            self.photo_paths[self.current_photo_index].clone(),
                            self.current_photo_index
                        ),
                        Message::PhotoLoaded
                    )
                } else {
                    Command::none()
                }
            }
            Message::SelectPhoto => {
                if let Some(photo) = self.cached_photos.get(&self.current_photo_index) {
                    self.selected_photos.insert(photo.path.clone(), true);
                    if self.current_photo_index < self.photo_paths.len() - 1 {
                        self.current_photo_index += 1;
                        self.cleanup_cache();
                        return Command::perform(
                            load_single_photo(
                                self.photo_paths[self.current_photo_index].clone(),
                                self.current_photo_index
                            ),
                            Message::PhotoLoaded
                        );
                    }
                }
                Command::none()
            }
            Message::DeletePhoto(index) => {
                if index < self.photo_paths.len() {
                    let path_to_delete = self.photo_paths[index].clone();
                    if self.link_raw_jpg && check_raw_exists(&path_to_delete) {
                        let raw_path = get_raw_path(&path_to_delete);
                        if let Err(e) = std::fs::remove_file(&raw_path) {
                            eprintln!("Failed to delete RAW file: {}", e);
                        }
                    }
                    if let Err(e) = std::fs::remove_file(&path_to_delete) {
                        eprintln!("Failed to delete JPG file: {}", e);
                    }
                    self.photo_paths.remove(index);
                    self.cached_photos.remove(&index);
                    if self.current_photo_index >= self.photo_paths.len() {
                        self.current_photo_index = self.photo_paths.len().saturating_sub(1);
                    }
                }
                Command::none()
            }
            Message::KeyPressed(event) => {
                match event {
                    keyboard::Event::KeyPressed { key_code, .. } => {
                        match key_code {
                            keyboard::KeyCode::Left => {
                                if self.current_photo_index > 0 {
                                    self.current_photo_index -= 1;
                                    self.cleanup_cache();
                                    return Command::perform(
                                        load_single_photo(
                                            self.photo_paths[self.current_photo_index].clone(),
                                            self.current_photo_index
                                        ),
                                        Message::PhotoLoaded
                                    );
                                }
                            }
                            keyboard::KeyCode::Right => {
                                if self.current_photo_index < self.photo_paths.len() - 1 {
                                    self.current_photo_index += 1;
                                    self.cleanup_cache();
                                    return Command::perform(
                                        load_single_photo(
                                            self.photo_paths[self.current_photo_index].clone(),
                                            self.current_photo_index
                                        ),
                                        Message::PhotoLoaded
                                    );
                                }
                            }
                            keyboard::KeyCode::S => {
                                if let Some(photo) = self.cached_photos.get(&self.current_photo_index) {
                                    self.selected_photos.insert(photo.path.clone(), true);
                                    if self.current_photo_index < self.photo_paths.len() - 1 {
                                        self.current_photo_index += 1;
                                        self.cleanup_cache();
                                        return Command::perform(
                                            load_single_photo(
                                                self.photo_paths[self.current_photo_index].clone(),
                                                self.current_photo_index
                                            ),
                                            Message::PhotoLoaded
                                        );
                                    }
                                }
                            }
                            keyboard::KeyCode::D => {
                                if self.current_photo_index < self.photo_paths.len() {
                                    let index = self.current_photo_index;
                                    return Command::perform(
                                        async move { index },
                                        Message::DeletePhoto
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if self.photo_paths.is_empty() {
            return column![
                text("No photos available").size(30),
            ]
            .into();
        }

        if let Some(photo) = self.cached_photos.get(&self.current_photo_index) {
            let photo_element = container(
                column![
                    iced_image(photo.handle.clone())
                        .width(Length::Fill)
                        .height(Length::Fill),
                    text(&photo.exif_data),
                    row![
                        button("S - Select").on_press(Message::SelectPhoto),
                        button("D - Delete").on_press(Message::DeletePhoto(self.current_photo_index)),
                    ]
                ]
                .spacing(10)
            )
            .padding(10);

            column![
                row![
                    button("← Previous").on_press(Message::PreviousPhoto),
                    button("Next →").on_press(Message::NextPhoto),
                ].spacing(20),
                photo_element,
            ]
            .spacing(20)
            .into()
        } else {
            column![
                text("Loading...").size(30),
            ]
            .into()
        }
    }
}

async fn load_photo_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let photos_dir = std::path::Path::new("photos");
    
    if !photos_dir.exists() {
        if let Err(e) = std::fs::create_dir(photos_dir) {
            eprintln!("Failed to create photos directory: {}", e);
            return paths;
        }
    }

    for entry in WalkDir::new(photos_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            match extension.to_str().unwrap().to_lowercase().as_str() {
                "jpg" | "jpeg" | "JPG" | "JPEG" => {
                    paths.push(path.to_path_buf());
                }
                _ => {}
            }
        }
    }
    paths
}

async fn load_single_photo(path: PathBuf, index: usize) -> (usize, Photo) {
    let exif_data = if let Ok(file) = std::fs::File::open(&path) {
        if let Ok(exif_reader) = exif::Reader::new().read_from_container(&mut std::io::BufReader::new(file)) {
            format_exif_data(&exif_reader)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let has_raw = check_raw_exists(&path);
    let handle = if let Ok(img) = image_open(&path) {
        iced::widget::image::Handle::from_pixels(
            img.width(),
            img.height(),
            img.to_rgba8().into_raw(),
        )
    } else {
        iced::widget::image::Handle::from_pixels(1, 1, vec![0, 0, 0, 255])
    };

    (index, Photo {
        path,
        exif_data,
        has_raw,
        handle,
    })
}

fn format_exif_data(exif: &exif::Exif) -> String {
    let mut result = String::new();
    
    if let Some(create_date) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        result.push_str(&format!("Date Taken: {}\n", create_date.display_value()));
    }
    
    if let Some(camera) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        result.push_str(&format!("Camera: {}\n", camera.display_value()));
    }
    
    if let Some(focal_length) = exif.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        result.push_str(&format!("Focal Length: {}\n", focal_length.display_value()));
    }

    if let Some(iso) = exif.get_field(exif::Tag::ISOSpeed, exif::In::PRIMARY) {
        result.push_str(&format!("ISO: {}\n", iso.display_value()));
    }

    if let Some(aperture) = exif.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        result.push_str(&format!("Aperture: f/{}\n", aperture.display_value()));
    }

    if let Some(exposure) = exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        result.push_str(&format!("Shutter Speed: {}\n", exposure.display_value()));
    }
    
    result
}

fn check_raw_exists(jpg_path: &std::path::Path) -> bool {
    let raw_path = get_raw_path(jpg_path);
    raw_path.exists()
}

fn get_raw_path(jpg_path: &std::path::Path) -> PathBuf {
    let mut raw_path = jpg_path.to_path_buf();
    raw_path.set_extension("raw");
    raw_path
}

fn main() -> iced::Result {
    let settings = Settings {
        window: window::Settings {
            size: (1600, 900),
            ..Default::default()
        },
        ..Default::default()
    };
    
    PhotoSelector::run(settings)
}