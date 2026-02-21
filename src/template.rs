use crate::parser::ParsedMedia;

pub fn render_template(format: &str, media: &ParsedMedia) -> String {
    let mut out = format.to_string();
    out = out.replace("{title}", media.title_guess.as_deref().unwrap_or(""));
    out = out.replace("{year}", &media.year_guess.map(|v| v.to_string()).unwrap_or_default());
    out = replace_padded(out, "season", media.season);
    out = replace_padded(out, "episode", media.episode);
    out = out.replace("{ext}", &media.ext);
    clean_spaces(&out)
}

fn replace_padded(mut text: String, key: &str, value: Option<u16>) -> String {
    let mut start = 0;
    loop {
        let needle = format!("{{{key}:");
        let Some(idx) = text[start..].find(&needle) else {
            break;
        };
        let abs_idx = start + idx;
        let end = text[abs_idx..].find('}').map(|i| abs_idx + i);
        let Some(end) = end else { break };
        let token = &text[abs_idx..=end];
        let width = token
            .trim_start_matches(&format!("{{{key}:"))
            .trim_end_matches('}')
            .parse::<usize>()
            .unwrap_or(0);
        let replacement = match value {
            Some(v) if width > 0 => format!("{:0width$}", v, width = width),
            Some(v) => v.to_string(),
            None => String::new(),
        };
        text.replace_range(abs_idx..=end, &replacement);
        start = abs_idx + replacement.len();
    }
    text.replace(&format!("{{{key}}}"), &value.map(|v| v.to_string()).unwrap_or_default())
}

fn clean_spaces(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{MediaKind, ParsedMedia};

    fn media() -> ParsedMedia {
        ParsedMedia {
            kind: MediaKind::Episode,
            title_guess: Some("My Show".into()),
            year_guess: Some(2024),
            season: Some(3),
            episode: Some(9),
            ext: "mkv".into(),
        }
    }

    #[test]
    fn template_15_cases() {
        let m = media();
        let cases = vec![
            ("{title}", "My Show"),
            ("{title} ({year})", "My Show (2024)"),
            ("S{season:02}E{episode:02}", "S03E09"),
            ("{title}.S{season}E{episode}", "My Show.S3E9"),
            ("{title}.{ext}", "My Show.mkv"),
            ("  {title}   {year} ", "My Show 2024"),
            ("{title}-{season:03}", "My Show-003"),
            ("{title}-{episode:04}", "My Show-0009"),
            ("{title} {season}", "My Show 3"),
            ("{title} {episode}", "My Show 9"),
            ("{title} {season:0}", "My Show 3"),
            ("{title} {year} {ext}", "My Show 2024 mkv"),
            ("{title} S{season:02} E{episode:02}", "My Show S03 E09"),
            ("{title}-{season:02}x{episode:02}", "My Show-03x09"),
            ("{title}-{year}-{ext}", "My Show-2024-mkv"),
        ];
        for (fmt, expected) in cases {
            assert_eq!(render_template(fmt, &m), expected);
        }

        let mut missing = media();
        missing.year_guess = None;
        missing.season = None;
        missing.episode = None;
        assert_eq!(render_template("{title} {year}", &missing), "My Show");
        assert_eq!(render_template("S{season:02}E{episode:02}", &missing), "SE");
    }
}
