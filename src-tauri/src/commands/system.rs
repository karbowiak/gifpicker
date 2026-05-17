use crate::commands::{CommandError, CommandResult};

#[tauri::command]
pub async fn open_url(url: String) -> CommandResult<()> {
    open::that(url).map_err(CommandError::Io)
}
