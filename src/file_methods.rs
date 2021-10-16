use crate::guards::TokenChecker;
use rocket::form::Form;
use rocket::fs::FileName;
use rocket::fs::{NamedFile, TempFile};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;

#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    size: u64,
}

#[get("/files")]
pub fn files(
    _token: TokenChecker,
    config: &State<crate::Config>,
) -> Result<Json<Vec<FileEntry>>, std::io::Error> {
    let paths = std::fs::read_dir(&config.file_directory)?;
    let mut files = Vec::with_capacity(32);
    for entry in paths {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            let name = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap_or_default()
                .to_string();
            let size = entry.metadata()?.len();
            files.push(FileEntry { name, size });
        }
    }
    Ok(Json(files))
}

#[derive(Responder)]
pub enum UploadResult {
    #[response(status = 200)]
    Ok(()),
    #[response(status = 500)]
    InternalError(()),
}

#[derive(FromForm)]
pub struct FileUpload<'f> {
    file: TempFile<'f>,
}

#[post("/upload", data = "<form>")]
pub async fn upload(
    mut form: Form<FileUpload<'_>>,
    _token: TokenChecker,
    config: &State<crate::Config>,
) -> UploadResult {
    let name = form.file.raw_name().unwrap();
    let path = format!(
        "{}/{}",
        &config.file_directory,
        name.dangerous_unsafe_unsanitized_raw()
    );
    if let Err(_) = form.file.persist_to(&path).await {
        return UploadResult::InternalError(());
    }
    return UploadResult::Ok(());
}

#[get("/file/<file..>")]
pub async fn download(
    file: std::path::PathBuf,
    _token: TokenChecker,
    config: &State<crate::Config>,
) -> Option<NamedFile> {
    NamedFile::open(std::path::Path::new(&config.file_directory).join(file))
        .await
        .ok()
}

#[delete("/file/<file..>")]
pub async fn delete_file(
    file: std::path::PathBuf,
    _token: TokenChecker,
    config: &State<crate::Config>,
) -> std::io::Result<()> {
    let path = std::path::Path::new(&config.file_directory).join(file);
    std::fs::remove_file(path)
}

#[derive(Deserialize)]
pub struct RenameRequest {
    new_name: String,
}

#[patch("/file/<file..>", data = "<name>")]
pub fn rename(
    file: std::path::PathBuf,
    name: Json<RenameRequest>,
    config: &State<crate::Config>,
    _token: TokenChecker,
) -> std::io::Result<()> {
    let path = std::path::Path::new(&config.file_directory).join(file);
    std::fs::rename(path, &name.new_name)
}
