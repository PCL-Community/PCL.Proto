#[derive(serde::Serialize, serde::Deserialize)]
pub enum DownloadSource {
    Official,
    BMCLApi,
}
