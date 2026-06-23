use crate::commands::settings::{read_settings_file, write_settings_file};
use crate::models::settings::AppSettings;
use tauri::{LogicalPosition, LogicalSize, Monitor, WebviewWindow};

/// モニタ情報から論理座標系の矩形一覧を構築
fn collect_monitor_rects(window: &WebviewWindow) -> Vec<MonitorRect> {
    window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m: &Monitor| {
            let scale = m.scale_factor();
            let pos = m.position().to_logical::<f64>(scale);
            let size = m.size().to_logical::<f64>(scale);
            MonitorRect {
                x: pos.x.round() as i32,
                y: pos.y.round() as i32,
                width: size.width.round() as i32,
                height: size.height.round() as i32,
            }
        })
        .collect()
}

/// 起動時に保存済みウィンドウ状態を復元する。
/// 常にサイズを適用し、位置は有効モニタ内の場合のみ適用する。
/// フルスクリーン・最大化は復元しない(常に通常ウィンドウ)。
pub fn restore_window_state(window: &WebviewWindow, settings: &AppSettings) {
    let ws = &settings.window_state;

    let _ = window.set_size(LogicalSize::new(ws.width as f64, ws.height as f64));

    if let (Some(x), Some(y)) = (ws.x, ws.y) {
        let monitors = collect_monitor_rects(window);
        if is_position_visible((x, y), &monitors) {
            let _ = window.set_position(LogicalPosition::new(x as f64, y as f64));
        }
    }
}

/// 終了時にウィンドウ状態を保存する。
/// フルスクリーン中または最大化中はスキップし、最後の通常ウィンドウ状態を保持する。
pub fn save_window_state(window: &WebviewWindow) {
    if window.is_fullscreen().unwrap_or(false) {
        return;
    }
    if window.is_maximized().unwrap_or(false) {
        return;
    }

    let pos = match window.outer_position() {
        Ok(p) => p,
        Err(_) => return,
    };
    let size = match window.outer_size() {
        Ok(s) => s,
        Err(_) => return,
    };

    let scale = match window.scale_factor() {
        Ok(s) => s,
        Err(_) => return,
    };
    let logical_pos = pos.to_logical::<f64>(scale);
    let logical_size = size.to_logical::<f64>(scale);

    if let Ok(mut settings) = read_settings_file() {
        settings.window_state.x = Some(logical_pos.x.round() as i32);
        settings.window_state.y = Some(logical_pos.y.round() as i32);
        settings.window_state.width = logical_size.width.round() as u32;
        settings.window_state.height = logical_size.height.round() as u32;
        let _ = write_settings_file(&settings);
    }
}

/// モニタの論理座標系矩形
#[derive(Debug, Clone, Copy)]
pub(crate) struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// 位置がいずれかのモニタ矩形内にあるか(画面外復元のガード用)
pub(crate) fn is_position_visible(pos: (i32, i32), monitors: &[MonitorRect]) -> bool {
    let (x, y) = pos;
    monitors
        .iter()
        .any(|m| x >= m.x && x < m.x + m.width && y >= m.y && y < m.y + m.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_inside_monitor() {
        let monitors = [MonitorRect { x: 0, y: 0, width: 1920, height: 1080 }];
        assert!(is_position_visible((100, 100), &monitors));
    }

    #[test]
    fn not_visible_outside_any_monitor() {
        let monitors = [MonitorRect { x: 0, y: 0, width: 1920, height: 1080 }];
        assert!(!is_position_visible((5000, 5000), &monitors));
    }

    #[test]
    fn visible_at_top_left_corner() {
        let monitors = [MonitorRect { x: 0, y: 0, width: 100, height: 100 }];
        assert!(is_position_visible((0, 0), &monitors));
    }

    #[test]
    fn not_visible_just_past_edge() {
        let monitors = [MonitorRect { x: 0, y: 0, width: 100, height: 100 }];
        assert!(!is_position_visible((100, 50), &monitors));
    }

    #[test]
    fn visible_in_second_monitor() {
        let monitors = [
            MonitorRect { x: 0, y: 0, width: 1920, height: 1080 },
            MonitorRect { x: 1920, y: 0, width: 1920, height: 1080 },
        ];
        assert!(is_position_visible((2000, 500), &monitors));
        assert!(!is_position_visible((4000, 500), &monitors));
    }

    #[test]
    fn visible_negative_coords_multi_monitor() {
        let monitors = [
            MonitorRect { x: -1920, y: 0, width: 1920, height: 1080 },
            MonitorRect { x: 0, y: 0, width: 1920, height: 1080 },
        ];
        assert!(is_position_visible((-1000, 500), &monitors));
    }

    #[test]
    fn not_visible_when_no_monitors() {
        assert!(!is_position_visible((100, 100), &[]));
    }
}
