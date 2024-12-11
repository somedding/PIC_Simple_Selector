use iced::{
    widget::{button, checkbox, column, container, row, text},
    Application, Command, Element, Settings, Theme,
};
use std::{path::PathBuf, collections::HashMap};
use walkdir::WalkDir;

struct PhotoSelector {
    photos: Vec<Photo>,
    link_raw_jpg: bool,
}

#[derive(Debug, Clone)]
struct Photo {
    path: PathBuf,
    exif_data: String,
    has_raw: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ToggleLinkRawJpg(bool),
    DeletePhoto(usize),
    LoadPhotos(Vec<Photo>),
}

impl Application for PhotoSelector {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            PhotoSelector {
                photos: Vec::new(),
                link_raw_jpg: true,
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
            Message::DeletePhoto(index) => {
                if let Some(photo) = self.photos.get(index) {
                    if self.link_raw_jpg && photo.has_raw {
                        // RAW 파일도 함께 삭제
                        let raw_path = get_raw_path(&photo.path);
                        if let Err(e) = std::fs::remove_file(&raw_path) {
                            eprintln!("Failed to delete RAW file: {}", e);
                        }
                    }
                    if let Err(e) = std::fs::remove_file(&photo.path) {
                        eprintln!("Failed to delete JPG file: {}", e);
                    }
                    self.photos.remove(index);
                }
                Command::none()
            }
            Message::LoadPhotos(photos) => {
                self.photos = photos;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut content = column![
            checkbox(
                "RAW+JPG 함께 삭제",
                self.link_raw_jpg,
                Message::ToggleLinkRawJpg
            )
        ];

        for (i, photo) in self.photos.iter().enumerate() {
            content = content.push(
                row![
                    text(&photo.path.to_string_lossy()),
                    text(&photo.exif_data),
                    button("삭제").on_press(Message::DeletePhoto(i))
                ]
                .spacing(20)
            );
        }

        container(content).padding(20).into()
    }
}

async fn load_photos() -> Vec<Photo> {
    let mut photos = Vec::new();
    let mut raw_files = HashMap::new();

    // "photos" 디렉토리에서 이미지 파일 검색
    let photos_dir = std::path::Path::new("photos");
    
    // photos 디렉토리가 없으면 생성
    if !photos_dir.exists() {
        if let Err(e) = std::fs::create_dir(photos_dir) {
            eprintln!("Failed to create photos directory: {}", e);
            return photos;
        }
    }

    // 수정된 부분: "." 대신 "photos" 디렉토리를 검색
    for entry in WalkDir::new(photos_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            match extension.to_str().unwrap().to_lowercase().as_str() {
                "jpg" | "jpeg" => {
                    if let Ok(file) = std::fs::File::open(path) {
                        if let Ok(exif_reader) = exif::Reader::new().read_from_container(&mut std::io::BufReader::new(file)) {
                            let exif_data = format_exif_data(&exif_reader);
                            let has_raw = check_raw_exists(path);
                            photos.push(Photo {
                                path: path.to_path_buf(),
                                exif_data,
                                has_raw,
                            });
                        }
                    }
                }
                "raw" | "cr2" | "nef" | "arw" => {
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

fn format_exif_data(exif: &exif::Exif) -> String {
    let mut result = String::new();
    
    if let Some(create_date) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        result.push_str(&format!("촬영일시: {}\n", create_date.display_value()));
    }
    
    if let Some(camera) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        result.push_str(&format!("카메라: {}\n", camera.display_value()));
    }
    
    if let Some(focal_length) = exif.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        result.push_str(&format!("초점거리: {}\n", focal_length.display_value()));
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
    PhotoSelector::run(Settings::default())
} 