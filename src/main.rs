use iced::{
    widget::{button, column, container, row, text, image as iced_image},
    Application, Command, Element, Settings, Theme, Length,
    executor, window,
};
use std::{path::PathBuf, collections::HashMap};
use walkdir::WalkDir;
use image::open as image_open;

struct PhotoSelector {
    photos: Vec<Photo>,
    link_raw_jpg: bool,
    selected_photos: HashMap<PathBuf, bool>,
    current_photo_index: usize,
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
    ToggleLinkRawJpg(bool),
    LoadPhotos(Vec<Photo>),
    ToggleSelect(PathBuf),
    DeleteSelected,
    NextPhoto,
    PreviousPhoto,
    SelectPhoto,
    DeletePhoto(usize),
}

impl Application for PhotoSelector {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            PhotoSelector {
                photos: Vec::new(),
                link_raw_jpg: true,
                selected_photos: HashMap::new(),
                current_photo_index: 0,
            },
            Command::perform(load_photos(), Message::LoadPhotos),
        )
    }

    fn title(&self) -> String {
        String::from("Photo Selector")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ToggleLinkRawJpg(value) => {
                self.link_raw_jpg = value;
                Command::none()
            }
            Message::LoadPhotos(photos) => {
                self.photos = photos;
                Command::none()
            }
            Message::ToggleSelect(path) => {
                let selected = self.selected_photos.entry(path).or_insert(false);
                *selected = !*selected;
                Command::none()
            }
            Message::DeleteSelected => {
                self.photos.retain(|photo| {
                    let should_keep = !self.selected_photos.get(&photo.path).unwrap_or(&false);
                    if !should_keep {
                        if self.link_raw_jpg && photo.has_raw {
                            let raw_path = get_raw_path(&photo.path);
                            if let Err(e) = std::fs::remove_file(&raw_path) {
                                eprintln!("Failed to delete RAW file: {}", e);
                            }
                        }
                        if let Err(e) = std::fs::remove_file(&photo.path) {
                            eprintln!("Failed to delete JPG file: {}", e);
                        }
                    }
                    should_keep
                });
                self.selected_photos.clear();
                Command::none()
            }
            Message::NextPhoto => {
                if self.current_photo_index < self.photos.len() - 1 {
                    self.current_photo_index += 1;
                }
                Command::none()
            }
            Message::PreviousPhoto => {
                if self.current_photo_index > 0 {
                    self.current_photo_index -= 1;
                }
                Command::none()
            }
            Message::SelectPhoto => {
                let photo = &self.photos[self.current_photo_index];
                self.selected_photos.insert(photo.path.clone(), true);
                if self.current_photo_index < self.photos.len() - 1 {
                    self.current_photo_index += 1;
                }
                Command::none()
            }
            Message::DeletePhoto(index) => {
                if index < self.photos.len() {
                    let photo_to_delete = self.photos[index].clone();
                    if self.link_raw_jpg && photo_to_delete.has_raw {
                        let raw_path = get_raw_path(&photo_to_delete.path);
                        if let Err(e) = std::fs::remove_file(&raw_path) {
                            eprintln!("Failed to delete RAW file: {}", e);
                        }
                    }
                    if let Err(e) = std::fs::remove_file(&photo_to_delete.path) {
                        eprintln!("Failed to delete JPG file: {}", e);
                    }
                    self.photos.remove(index);
                    self.selected_photos.remove(&photo_to_delete.path);
                    if self.current_photo_index >= self.photos.len() {
                        self.current_photo_index = self.photos.len().saturating_sub(1);
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if self.photos.is_empty() {
            return column![
                text("No photos available").size(30),
            ]
            .into();
        }

        let photo = &self.photos[self.current_photo_index];
        let photo_element = container(
            column![
                iced_image(photo.handle.clone())
                    .width(Length::Fixed(600.0))
                    .height(Length::Fixed(400.0)),
                text(&photo.exif_data),
                row![
                    button("Select").on_press(Message::SelectPhoto),
                    button("Delete").on_press(Message::DeletePhoto(self.current_photo_index)),
                ]
            ]
            .spacing(10)
        )
        .padding(10);

        let controls = row![
            button("Previous").on_press(Message::PreviousPhoto),
            button("Next").on_press(Message::NextPhoto),
        ].spacing(20);

        column![
            controls,
            photo_element,
        ]
        .spacing(20)
        .into()
    }
}

async fn load_photos() -> Vec<Photo> {
    let mut photos = Vec::new();
    let mut raw_files = HashMap::new();

    let photos_dir = std::path::Path::new("photos");
    
    if !photos_dir.exists() {
        if let Err(e) = std::fs::create_dir(photos_dir) {
            eprintln!("Failed to create photos directory: {}", e);
            return photos;
        }
    }

    for entry in WalkDir::new(photos_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        println!("Checking file: {:?}", path);
        if let Some(extension) = path.extension() {
            match extension.to_str().unwrap().to_lowercase().as_str() {
                "jpg" | "jpeg" | "JPG" | "JPEG" => {
                    println!("Found JPG file: {:?}", path);
                    if let Ok(file) = std::fs::File::open(&path) {
                        if let Ok(exif_reader) = exif::Reader::new().read_from_container(&mut std::io::BufReader::new(file)) {
                            let exif_data = format_exif_data(&exif_reader);
                            let has_raw = check_raw_exists(path);
                            
                            if let Ok(img) = image_open(&path) {
                                println!("Successfully opened image: {:?}", path);
                                let handle = iced::widget::image::Handle::from_pixels(
                                    img.width(),
                                    img.height(),
                                    img.to_rgba8().into_raw(),
                                );
                                
                                photos.push(Photo {
                                    path: path.to_path_buf(),
                                    exif_data,
                                    has_raw,
                                    handle,
                                });
                            } else {
                                println!("Failed to open image: {:?}", path);
                            }
                        } else {
                            println!("Failed to read EXIF data for: {:?}", path);
                        }
                    } else {
                        println!("Failed to open file: {:?}", path);
                    }
                }
                "raw" | "cr2" | "nef" | "arw" | "orf" | "rw2" | "dng" | "CR2" => {
                    raw_files.insert(
                        path.file_stem().unwrap().to_str().unwrap().to_string(),
                        path.to_path_buf(),
                    );
                }
                _ => {}
            }
        }
    }

    photos
}

#[derive(Debug, Clone, Copy)]
struct SelectedStyle;

impl container::StyleSheet for SelectedStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(0.0, 0.5, 1.0, 0.1)
            )),
            border_radius: 5.0.into(),
            border_width: 2.0,
            border_color: iced::Color::from_rgba(0.0, 0.5, 1.0, 1.0),
            ..Default::default()
        }
    }
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