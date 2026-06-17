use soundworks_core::AppOverview;

#[tauri::command]
fn get_app_overview() -> AppOverview {
    app_overview()
}

pub fn app_overview() -> AppOverview {
    AppOverview::baseline()
}

pub fn builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_app_overview])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    builder()
        .run(tauri::generate_context!())
        .expect("failed to run SoundWorks desktop app");
}

#[cfg(test)]
mod tests {
    use super::app_overview;

    #[test]
    fn app_overview_command_returns_soundworks() {
        let overview = app_overview();

        assert_eq!(overview.product_name, "SoundWorks");
        assert_eq!(overview.commands[0].name, "get_app_overview");
    }
}
