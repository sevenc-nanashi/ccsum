use colored::Colorize;
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

pub fn colorize_checksum(checksum_display: &str, checksum: &[u8], dim: bool) -> String {
    let (start, end) = checksum_to_gradient_colors(checksum, dim);
    let chars: Vec<char> = checksum_display.chars().collect();
    let len = chars.len();
    if len == 0 {
        return String::new();
    }

    let mut out = String::new();
    for (index, ch) in chars.into_iter().enumerate() {
        let t = if len > 1 {
            index as f32 / (len - 1) as f32
        } else {
            0.0
        };
        let r = mix_channel(start.red, end.red, t);
        let g = mix_channel(start.green, end.green, t);
        let b = mix_channel(start.blue, end.blue, t);
        out.push_str(
            &ch.to_string()
                .color(colored::Color::TrueColor { r, g, b })
                .to_string(),
        );
    }

    out
}

fn checksum_to_gradient_colors(
    checksum: &[u8],
    dim: bool,
) -> (palette::Srgb<f32>, palette::Srgb<f32>) {
    let hue_start = bytes_to_hue(checksum[0], checksum[1]);
    let hue_end = bytes_to_hue(checksum[checksum.len() - 1], checksum[checksum.len() - 2]);
    let (lightness, chroma) = if dim { (0.5, 0.1) } else { (0.7, 0.4) };
    let start: palette::Srgb<f32> =
        palette::oklch::Oklch::new(lightness, chroma, hue_start).into_color();
    let end: palette::Srgb<f32> =
        palette::oklch::Oklch::new(lightness, chroma, hue_end).into_color();

    (start, end)
}

fn bytes_to_hue(high: u8, low: u8) -> f32 {
    let combined = ((high as u16) << 8) | (low as u16);
    let index = crate::table::RAND_TABLE[combined as usize];
    index as f32 * (360.0 / 65536.0)
}

fn mix_channel(start: f32, end: f32, t: f32) -> u8 {
    let value = start + (end - start) * t;
    let value = value.clamp(0.0, 1.0) * 255.0;
    value.round().clamp(0.0, 255.0) as u8
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
