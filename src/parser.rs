use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaKind {
    Movie,
    Episode,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMedia {
    pub kind: MediaKind,
    pub title_guess: Option<String>,
    pub year_guess: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub ext: String,
}

pub fn parse_media(path: &Path) -> ParsedMedia {
    let file_name = path.file_name().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
    let (raw, ext) = split_name_ext(&file_name);
    let stem = raw.replace(['.', '_', '-'], " ").split_whitespace().collect::<Vec<_>>().join(" ");

    if let Some((idx, s, e)) = find_episode(&stem) {
        return ParsedMedia {
            kind: MediaKind::Episode,
            title_guess: (!stem[..idx].trim().is_empty()).then(|| stem[..idx].trim().to_string()),
            year_guess: None,
            season: Some(s),
            episode: Some(e),
            ext,
        };
    }

    if let Some((idx, year)) = find_year(&stem) {
        return ParsedMedia {
            kind: MediaKind::Movie,
            title_guess: (!stem[..idx].trim().is_empty()).then(|| stem[..idx].trim().to_string()),
            year_guess: Some(year),
            season: None,
            episode: None,
            ext,
        };
    }

    ParsedMedia { kind: MediaKind::Unknown, title_guess: None, year_guess: None, season: None, episode: None, ext }
}

fn split_name_ext(file_name: &str) -> (String, String) {
    if let Some((name, ext)) = file_name.rsplit_once('.') {
        if ext.chars().all(|c| c.is_ascii_alphabetic()) && ext.len() <= 5 {
            return (name.to_string(), ext.to_string());
        }
    }
    (file_name.to_string(), String::new())
}

fn find_episode(s: &str) -> Option<(usize, u16, u16)> {
    let b = s.as_bytes();
    for i in 0..b.len() {
        if (b[i] == b'S' || b[i] == b's') && i + 3 < b.len() {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() && j - i <= 2 { j += 1; }
            if j < b.len() && (b[j] == b'E' || b[j] == b'e') {
                let mut k = j + 1;
                while k < b.len() && b[k].is_ascii_digit() && k - j <= 3 { k += 1; }
                if j > i + 1 && k > j + 1 {
                    let season = s[i + 1..j].parse().ok()?;
                    let episode = s[j + 1..k].parse().ok()?;
                    return Some((i, season, episode));
                }
            }
        }
        if b[i].is_ascii_digit() {
            let mut j = i;
            while j < b.len() && b[j].is_ascii_digit() && j - i < 2 { j += 1; }
            if j < b.len() && (b[j] == b'x' || b[j] == b'X') {
                let mut k = j + 1;
                while k < b.len() && b[k].is_ascii_digit() && k - j <= 3 { k += 1; }
                if k > j + 1 {
                    let season = s[i..j].parse().ok()?;
                    let episode = s[j + 1..k].parse().ok()?;
                    return Some((i, season, episode));
                }
            }
        }
    }
    None
}

fn find_year(s: &str) -> Option<(usize, u16)> {
    for (idx, _) in s.char_indices() {
        if idx + 4 <= s.len() {
            let part = &s[idx..idx + 4];
            if part.chars().all(|c| c.is_ascii_digit()) {
                let y: u16 = part.parse().ok()?;
                if (1900..=2100).contains(&y)
                    && (idx == 0 || !s.as_bytes()[idx - 1].is_ascii_digit())
                    && (idx + 4 == s.len() || !s.as_bytes()[idx + 4].is_ascii_digit())
                {
                    return Some((idx, y));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn p(name: &str) -> ParsedMedia { parse_media(&PathBuf::from(name)) }

    #[test]
    fn parser_30_cases() {
        let cases = vec![
            ("Der.Teufel.traegt.Prada.2006.German.AC3.DL.1080p.BluRay.x265-FuN.mkv", MediaKind::Movie),
            ("Movie.Title.1999.1080p.mkv", MediaKind::Movie),("Movie_Title_2010.mp4", MediaKind::Movie),("A.B.C.2023.avi", MediaKind::Movie),
            ("No.Year.Here.mkv", MediaKind::Unknown),("Show.S03E09.mkv", MediaKind::Episode),("Show.S3E090.mkv", MediaKind::Episode),
            ("Show.3x90.mkv", MediaKind::Episode),("Show_12x3.mp4", MediaKind::Episode),("abcS01E01xyz.mkv", MediaKind::Episode),
            ("Title.1980.mkv", MediaKind::Movie),("Title.2100.mkv", MediaKind::Movie),("Title.1899.mkv", MediaKind::Unknown),
            ("s01e01.mkv", MediaKind::Episode),("S01E01.mkv", MediaKind::Episode),("random.txt", MediaKind::Unknown),
            ("Film-2001-final.mov", MediaKind::Movie),("Serie-1x2-final.mov", MediaKind::Episode),("Serie-S10E02-final.mov", MediaKind::Episode),
            ("Show.S00E01.mkv", MediaKind::Episode),("Show.0x1.mkv", MediaKind::Episode),("Movie.2000.Special.Edition.mkv", MediaKind::Movie),
            ("Movie.2000.2001.mkv", MediaKind::Movie),("Just.Name", MediaKind::Unknown),(".hiddenfile", MediaKind::Unknown),
            ("Edge.S1E1.mkv", MediaKind::Episode),("Edge.1x001.mkv", MediaKind::Episode),("Edge.2005", MediaKind::Movie),
            ("Edge_2015_test.flac", MediaKind::Movie),("no_ext_S02E03", MediaKind::Episode),
        ];
        for (n, k) in cases { assert_eq!(p(n).kind, k, "{n}"); }
        let m = p("Der.Teufel.traegt.Prada.2006.German.AC3.DL.1080p.BluRay.x265-FuN.mkv");
        assert_eq!(m.title_guess.as_deref(), Some("Der Teufel traegt Prada"));
        assert_eq!(m.year_guess, Some(2006));
        let e = p("Show.S03E090.mkv");
        assert_eq!(e.season, Some(3));
        assert_eq!(e.episode, Some(90));
    }
}
