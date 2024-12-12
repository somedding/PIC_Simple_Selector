use iced::{
    widget::{button, column, container, row, text, image as iced_image},
    Application, Command, Element, Settings, Theme, Length,
    executor, window, keyboard, event, subscription,
};
use std::{path::PathBuf, collections::HashMap};
use walkdir::WalkDir;
use image::open as image_open;
use rayon::prelude::*;
use rfd;

struct PhotoSelector {
    photo_paths: Vec<PathBuf>,
    cached_photos: HashMap<usize, Photo>,
    selected_photos: HashMap<PathBuf, bool>,
    current_photo_index: usize,
    loading_file: Option<String>,
}

impl PhotoSelector {
    fn cleanup_cache(&mut self) {
        let keep_indices: Vec<usize> = vec![
            self.current_photo_index.saturating_sub(1),
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
    handle: iced::widget::image::Handle,
}

#[derive(Debug, Clone)]
enum Message {
    LoadPhotoPaths(Vec<PathBuf>),
    PhotoLoaded((usize, Photo)),
    NextPhoto,
    PreviousPhoto,
    SelectPhoto,
    DeletePhoto(usize),
    KeyPressed(keyboard::Event),
    LoadingStatus(String),
    OpenFolderDialog,
    FolderSelected(PathBuf),
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
                selected_photos: HashMap::new(),
                current_photo_index: 0,
                loading_file: None,
            },
            Command::perform(
                async {
                    if let Some(folder) = rfd::AsyncFileDialog::new()
                        .set_title("Select Photos Directory")
                        .pick_folder()
                        .await {
                        Some(folder.path().to_path_buf())
                    } else {
                        None
                    }
                },
                |path| path.map_or(Message::LoadPhotoPaths(Vec::new()), Message::FolderSelected)
            ),
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
                    Command::batch(
                        self.photo_paths.iter()
                            .take(10)
                            .enumerate()
                            .map(|(i, path)| {
                                let filename = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                Command::batch(vec![
                                    Command::perform(
                                        async move { filename.clone() },
                                        Message::LoadingStatus
                                    ),
                                    Command::perform(
                                        load_single_photo(path.clone(), i),
                                        Message::PhotoLoaded
                                    )
                                ])
                            })
                            .collect::<Vec<_>>()
                    )
                } else {
                    Command::none()
                }
            }
            Message::PhotoLoaded((index, photo)) => {
                self.cached_photos.insert(index, photo);
                Command::none()
            }
            Message::NextPhoto => {
                if self.current_photo_index < self.photo_paths.len() - 1 {
                    self.current_photo_index += 1;
                    self.cleanup_cache();

                    let current_batch = self.current_photo_index / 10;
                    let position_in_batch = self.current_photo_index % 10;
                    
                    if position_in_batch >= 5 {
                        let next_batch_start = (current_batch + 1) * 10;
                        
                        if next_batch_start < self.photo_paths.len() && 
                           !self.cached_photos.contains_key(&next_batch_start) {
                            return Command::batch(
                                self.photo_paths.iter()
                                    .skip(next_batch_start)
                                    .take(10)
                                    .enumerate()
                                    .map(|(i, path)| {
                                        let actual_index = next_batch_start + i;
                                        let filename = path.file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("unknown")
                                            .to_string();
                                        Command::batch(vec![
                                            Command::perform(
                                                async move { filename.clone() },
                                                Message::LoadingStatus
                                            ),
                                            Command::perform(
                                                load_single_photo(path.clone(), actual_index),
                                                Message::PhotoLoaded
                                            )
                                        ])
                                    })
                                    .collect::<Vec<_>>()
                            );
                        }
                    }
                }
                Command::none()
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
            Message::LoadingStatus(filename) => {
                self.loading_file = Some(filename);
                Command::none()
            }
            Message::OpenFolderDialog => {
                Command::perform(
                    async {
                        if let Some(folder) = rfd::AsyncFileDialog::new()
                            .set_title("Select Photos Directory")
                            .pick_folder()
                            .await {
                            Some(folder.path().to_path_buf())
                        } else {
                            None
                        }
                    },
                    |path| path.map_or(Message::LoadPhotoPaths(Vec::new()), Message::FolderSelected)
                )
            }
            Message::FolderSelected(folder_path) => {
                Command::perform(
                    load_photo_paths_from(folder_path),
                    Message::LoadPhotoPaths
                )
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if self.photo_paths.is_empty() {
            return column![
                text("No photos available").size(30),
                button("Select Folder").on_press(Message::OpenFolderDialog),
            ]
            .spacing(20)
            .into();
        }

        if let Some(photo) = self.cached_photos.get(&self.current_photo_index) {
            let total_photos = self.photo_paths.len();
            let current_index = self.current_photo_index + 1;

            let photo_element = container(
                column![
                    iced_image(photo.handle.clone())
                        .width(Length::Fill)
                        .height(Length::Fill),
                    text(&photo.exif_data),
                    row![
                        button("S - Select").on_press(Message::SelectPhoto),
                        button("D - Delete").on_press(Message::DeletePhoto(self.current_photo_index)),
                    ],
                    text(format!("{}/{}", current_index, total_photos)).size(20),
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
            let loading_text = if let Some(filename) = &self.loading_file {
                format!("Loading: {}\n({} photos loaded)", 
                    filename, 
                    self.cached_photos.len()
                )
            } else {
                format!("Loading...\n({} photos loaded)",
                    self.cached_photos.len()
                )
            };

            column![
                text(loading_text).size(30),
            ]
            .into()
        }
    }
}

async fn load_single_photo(path: PathBuf, index: usize) -> (usize, Photo) {
    let _filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let exif_data = if let Ok(file) = std::fs::File::open(&path) {
        if let Ok(exif_reader) = exif::Reader::new().read_from_container(&mut std::io::BufReader::new(file)) {
            format_exif_data(&exif_reader)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let handle = if let Ok(img) = image_open(&path) {
        let (width, height) = (img.width(), img.height());
        let handle = if width > 1600 || height > 900 {
            let aspect_ratio = width as f32 / height as f32;
            
            let (new_width, new_height) = if aspect_ratio < 1.0 {
                let new_height = 900;
                let new_width = (new_height as f32 * aspect_ratio) as u32;
                (new_width, new_height)
            } else {
                let new_width = 1600;
                let new_height = (new_width as f32 / aspect_ratio) as u32;
                (new_width, new_height)
            };

            let resized = img.resize_exact(
                new_width,
                new_height,
                image::imageops::FilterType::Triangle
            );
            iced::widget::image::Handle::from_pixels(
                resized.width(),
                resized.height(),
                resized.to_rgba8().into_raw(),
            )
        } else {
            iced::widget::image::Handle::from_pixels(
                width,
                height,
                img.to_rgba8().into_raw(),
            )
        };
        handle
    } else {
        iced::widget::image::Handle::from_pixels(1, 1, vec![0, 0, 0, 255])
    };

    (index, Photo {
        path,
        exif_data,
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

async fn load_photo_paths_from(folder_path: PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
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

    paths.par_sort();
    paths
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