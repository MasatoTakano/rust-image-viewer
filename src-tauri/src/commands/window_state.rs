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
