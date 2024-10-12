mod api;
mod responses;

use color_eyre::Result;

use self::responses::SeriesResource;
pub use self::responses::SeriesStatus;

pub async fn get_sonarr_data(id: i32, is_4k: bool) -> Result<SeriesResource> {
    let path = format!("/series/{}", id);
    api::get(&path, None, is_4k).await
}

pub async fn remove_sonarr_data_and_files(sonarr_id: i32) -> Result<()> {
    let path = format!("/series/{}", sonarr_id);
    let params = vec![("deleteFiles", "true"), ("addImportListExclusion", "false")];
    api::delete(path.as_str(), Some(params)).await
}
