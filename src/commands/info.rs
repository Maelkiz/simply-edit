use std::fs;
use std::io::BufReader;
use std::path::Path;

use image::GenericImageView;

pub(crate) fn run_info(path: &str) -> Result<(), String> {
    let meta =
        fs::metadata(path).map_err(|e| format!("info: failed to read '{path}': {e}"))?;

    let reader = image::ImageReader::open(path)
        .map_err(|e| format!("info: failed to open '{path}': {e}"))?
        .with_guessed_format()
        .map_err(|e| format!("info: failed to read '{path}': {e}"))?;

    let format_name = format_display_name(reader.format());

    let img = reader
        .decode()
        .map_err(|e| format!("info: failed to decode '{path}': {e}"))?;

    let (width, height) = img.dimensions();
    let (space, depth, alpha) = color_info(img.color());

    let filename = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());

    let (has_exif, created) = read_exif(path);
    let icc_profile = read_icc_profile(path);

    println!("File: {filename}");
    println!("Format: {format_name}");
    println!("Dimensions: {width}\u{00d7}{height}");
    println!("Size: {}", format_size(meta.len()));
    println!();
    println!("Color:");
    println!("  Space: {space}");
    println!("  Depth: {depth}-bit");
    println!("  Alpha: {}", if alpha { "yes" } else { "no" });
    println!();
    println!("Metadata:");
    println!("  EXIF: {}", if has_exif { "yes" } else { "no" });
    match &icc_profile {
        Some(name) => println!("  ICC profile: {name}"),
        None => println!("  ICC profile: none"),
    }
    if let Some(date) = created {
        println!();
        println!("Created: {date}");
    }

    Ok(())
}

fn format_display_name(format: Option<image::ImageFormat>) -> &'static str {
    match format {
        Some(image::ImageFormat::Jpeg) => "JPEG",
        Some(image::ImageFormat::Png) => "PNG",
        Some(image::ImageFormat::WebP) => "WebP",
        Some(image::ImageFormat::Gif) => "GIF",
        Some(image::ImageFormat::Bmp) => "BMP",
        Some(image::ImageFormat::Tiff) => "TIFF",
        Some(image::ImageFormat::Ico) => "ICO",
        _ => "Unknown",
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

fn color_info(color_type: image::ColorType) -> (&'static str, u8, bool) {
    use image::ColorType::*;
    match color_type {
        L8 => ("Grayscale", 8, false),
        La8 => ("Grayscale", 8, true),
        L16 => ("Grayscale", 16, false),
        La16 => ("Grayscale", 16, true),
        Rgb8 => ("sRGB", 8, false),
        Rgba8 => ("sRGB", 8, true),
        Rgb16 => ("sRGB", 16, false),
        Rgba16 => ("sRGB", 16, true),
        Rgb32F => ("Linear RGB", 32, false),
        Rgba32F => ("Linear RGB", 32, true),
        _ => ("Unknown", 8, false),
    }
}

fn read_exif(path: &str) -> (bool, Option<String>) {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (false, None),
    };
    match exif::Reader::new().read_from_container(&mut BufReader::new(file)) {
        Ok(exif) => {
            let date = exif
                .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
                .map(|f| f.display_value().to_string());
            (true, date)
        }
        Err(_) => (false, None),
    }
}

fn read_icc_profile(path: &str) -> Option<String> {
    use std::io::Read;
    let file = fs::File::open(path).ok()?;
    let mut data = Vec::with_capacity(65536);
    file.take(262144).read_to_end(&mut data).ok()?;
    if data.starts_with(b"\xff\xd8") {
        read_icc_from_jpeg(&data)
    } else if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        read_icc_from_png(&data)
    } else {
        None
    }
}

fn read_icc_from_jpeg(data: &[u8]) -> Option<String> {
    let mut segments: Vec<(u8, Vec<u8>)> = Vec::new();
    let mut pos = 2; // skip SOI
    while pos + 3 < data.len() {
        if data[pos] != 0xFF {
            break;
        }
        let marker = data[pos + 1];
        if marker == 0xDA {
            break; // SOS — start of scan data
        }
        let seg_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        if marker == 0xE2 && pos + 18 <= data.len() {
            let id_end = pos + 4 + 12;
            if id_end <= data.len() && &data[pos + 4..id_end] == b"ICC_PROFILE\0" {
                let seq_num = data[pos + 16];
                let start = pos + 18;
                let end = (pos + 2 + seg_len).min(data.len());
                if start < end {
                    segments.push((seq_num, data[start..end].to_vec()));
                }
            }
        }
        pos += 2 + seg_len;
    }
    if segments.is_empty() {
        return None;
    }
    segments.sort_by_key(|(seq, _)| *seq);
    let icc_data: Vec<u8> = segments.into_iter().flat_map(|(_, b)| b).collect();
    parse_icc_description(&icc_data)
}

fn read_icc_from_png(data: &[u8]) -> Option<String> {
    let mut pos = 8; // skip PNG signature
    while pos + 8 < data.len() {
        let chunk_len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let chunk_type = &data[pos + 4..pos + 8];
        if chunk_type == b"iCCP" {
            let data_end = (pos + 8 + chunk_len).min(data.len());
            let chunk_data = &data[pos + 8..data_end];
            let null_pos = chunk_data.iter().position(|&b| b == 0)?;
            let name = std::str::from_utf8(&chunk_data[..null_pos]).ok()?;
            return if name.is_empty() { None } else { Some(name.to_string()) };
        }
        if chunk_type == b"IDAT" {
            break;
        }
        pos += 8 + chunk_len + 4; // length + type + data + CRC
    }
    None
}

fn parse_icc_description(icc: &[u8]) -> Option<String> {
    if icc.len() < 132 {
        return None;
    }
    let tag_count =
        u32::from_be_bytes([icc[128], icc[129], icc[130], icc[131]]) as usize;

    for i in 0..tag_count {
        let entry = 132 + i * 12;
        if entry + 12 > icc.len() {
            break;
        }
        if &icc[entry..entry + 4] != b"desc" {
            continue;
        }
        let offset = u32::from_be_bytes([icc[entry + 4], icc[entry + 5], icc[entry + 6], icc[entry + 7]]) as usize;
        let size = u32::from_be_bytes([icc[entry + 8], icc[entry + 9], icc[entry + 10], icc[entry + 11]]) as usize;
        if offset + size > icc.len() || size < 12 {
            return None;
        }
        let tag = &icc[offset..offset + size];

        if tag.starts_with(b"mluc") {
            return parse_mluc(tag);
        }
        if tag.starts_with(b"desc") {
            // ICC v2 textDescriptionType: ASCII count at +8, string at +12
            if size < 12 {
                return None;
            }
            let ascii_count = u32::from_be_bytes([tag[8], tag[9], tag[10], tag[11]]) as usize;
            if ascii_count == 0 || 12 + ascii_count > size {
                return None;
            }
            let s = std::str::from_utf8(&tag[12..12 + ascii_count.saturating_sub(1)]).ok()?;
            return if s.is_empty() { None } else { Some(s.to_string()) };
        }
    }
    None
}

fn parse_mluc(tag: &[u8]) -> Option<String> {
    // multiLocalizedUnicodeType: 8-byte header, 4-byte count, 4-byte record_size, then records
    if tag.len() < 16 {
        return None;
    }
    let record_count = u32::from_be_bytes([tag[8], tag[9], tag[10], tag[11]]) as usize;

    for i in 0..record_count {
        let rec = 16 + i * 12;
        if rec + 12 > tag.len() {
            break;
        }
        let str_len = u32::from_be_bytes([tag[rec + 4], tag[rec + 5], tag[rec + 6], tag[rec + 7]]) as usize;
        let str_off = u32::from_be_bytes([tag[rec + 8], tag[rec + 9], tag[rec + 10], tag[rec + 11]]) as usize;
        if str_off + str_len > tag.len() || !str_len.is_multiple_of(2) {
            continue;
        }
        let utf16: Vec<u16> = tag[str_off..str_off + str_len]
            .chunks_exact(2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .collect();
        if let Ok(s) = String::from_utf16(&utf16)
            && !s.is_empty()
        {
            return Some(s);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn test_color_info_rgb8() {
        let (space, depth, alpha) = color_info(image::ColorType::Rgb8);
        assert_eq!(space, "sRGB");
        assert_eq!(depth, 8);
        assert!(!alpha);
    }

    #[test]
    fn test_color_info_rgba8() {
        let (space, depth, alpha) = color_info(image::ColorType::Rgba8);
        assert_eq!(space, "sRGB");
        assert_eq!(depth, 8);
        assert!(alpha);
    }

    #[test]
    fn test_color_info_grayscale() {
        let (space, depth, alpha) = color_info(image::ColorType::L8);
        assert_eq!(space, "Grayscale");
        assert_eq!(depth, 8);
        assert!(!alpha);
    }

    #[test]
    fn test_run_info_missing_file() {
        let result = run_info("/nonexistent/path/image.png");
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("info:"));
    }
}
