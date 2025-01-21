use palette::IntoColor;
use regex_split::RegexSplit;

pub fn split_at_last_segments(path: &str, n: usize) -> (Option<String>, String) {
    let mut segments = if cfg!(windows) {
        lazy_regex::regex!(r"\\")
    } else {
        lazy_regex::regex!(r"/")
    }
    .split_inclusive(path)
    .collect::<Vec<&str>>();
    let len = segments.len();
    if len <= n {
        return (None, path.to_string());
    }

    let last = segments.split_off(len - n);
    (Some(segments.join("")), last.join(""))
}

pub fn checksum_to_color(checksum: &[u8], dim: bool) -> (u8, u8, u8) {
    let hue = ((checksum[0] as f32) * 256.0 + checksum[1] as f32) / (256.0 * 256.0) * 360.0;
    let color = if dim {
        palette::oklch::Oklch::new(0.5, 0.1, hue)
    } else {
        palette::oklch::Oklch::new(0.7, 0.4, hue)
    };
    let rgb: palette::Srgb<f32> = color.into_color();
    let rgb: palette::Srgb<u8> = rgb.into_format();

    (rgb.red, rgb.green, rgb.blue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_last_segments() {
        let path = "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z";
        let (head, tail) = split_at_last_segments(path, 3);
        assert_eq!(
            head,
            Some("/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/".to_string())
        );
        assert_eq!(tail, "x/y/z");
    }

    #[test]
    fn test_extract_last_segments_exceed() {
        let path = "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z";
        let (head, tail) = split_at_last_segments(path, 30);
        assert_eq!(head, None);
        assert_eq!(tail, path);
    }
}
