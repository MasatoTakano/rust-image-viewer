/// 文字列を自然順（Natural Sort）で比較するためのキーを生成する。
/// 数字部分は数値として比較し、それ以外は文字列として比較する。
/// 例: "001.jpg" < "2.jpg" < "10.jpg" < "100.jpg"
pub fn natural_sort_key(s: &str) -> Vec<NaturalSegment> {
    let mut segments = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            let num_val: u64 = num_str.trim_start_matches('0').parse().unwrap_or(0);
            segments.push(NaturalSegment::Number(num_val));
        } else {
            segments.push(NaturalSegment::Text(ch.to_lowercase().collect::<String>()));
            chars.next();
        }
    }

    segments
}

/// 自然順ソート用のセグメント
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NaturalSegment {
    /// 数値セグメント（数値として比較）
    Number(u64),
    /// テキストセグメント（大文字小文字無視で比較）
    Text(String),
}

/// パス文字列のスライスを自然順でソートする
pub fn sort_paths_naturally(paths: &mut [String]) {
    paths.sort_by_cached_key(|s| natural_sort_key(s));
}

/// (path, display_name) タプルのスライスを path の自然順でソートする
pub fn sort_entries_by_path(entries: &mut [(String, String)]) {
    entries.sort_by_cached_key(|(path, _)| natural_sort_key(path));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_sort() {
        let mut paths = vec![
            "100.jpg".to_string(),
            "10.jpg".to_string(),
            "2.jpg".to_string(),
            "001.jpg".to_string(),
            "20.jpg".to_string(),
        ];
        sort_paths_naturally(&mut paths);
        assert_eq!(
            paths,
            vec![
                "001.jpg".to_string(),
                "2.jpg".to_string(),
                "10.jpg".to_string(),
                "20.jpg".to_string(),
                "100.jpg".to_string(),
            ]
        );
    }

    #[test]
    fn test_natural_sort_with_folders() {
        let mut paths = vec![
            "chapter10/page2.jpg".to_string(),
            "chapter1/page10.jpg".to_string(),
            "chapter1/page2.jpg".to_string(),
            "chapter2/page1.jpg".to_string(),
            "chapter1/page1.jpg".to_string(),
        ];
        sort_paths_naturally(&mut paths);
        assert_eq!(
            paths,
            vec![
                "chapter1/page1.jpg".to_string(),
                "chapter1/page2.jpg".to_string(),
                "chapter1/page10.jpg".to_string(),
                "chapter2/page1.jpg".to_string(),
                "chapter10/page2.jpg".to_string(),
            ]
        );
    }
}
