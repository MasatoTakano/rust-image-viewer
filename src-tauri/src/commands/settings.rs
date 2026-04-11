use crate::models::settings::AppSettings;

/// 設定ファイルのパスを取得する
fn get_settings_path() -> Result<std::path::PathBuf, String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "設定ディレクトリのパスを取得できません".to_string())?;
    let app_dir = config_dir.join("rust-image-viewer");
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("設定ディレクトリの作成に失敗: {}", e))?;
    Ok(app_dir.join("settings.json"))
}

/// 設定を読み込む。ファイルが存在しない場合はデフォルト設定を返す
#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    let path = get_settings_path()?;

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("設定ファイルの読み取りに失敗: {}", e))?;

    let settings: AppSettings = serde_json::from_str(&content).map_err(|e| {
        format!(
            "設定ファイルのパースに失敗: {}。デフォルト設定を使用してください。",
            e
        )
    })?;

    Ok(settings)
}

/// 設定を保存する
#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path()?;

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("設定のシリアライズに失敗: {}", e))?;

    std::fs::write(&path, content).map_err(|e| format!("設定ファイルの書き込みに失敗: {}", e))?;

    Ok(())
}
