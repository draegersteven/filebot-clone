use crate::cli::Action;
use crate::matcher::{match_movie, HttpClient, MovieQuery};
use crate::parser::{parse_media, MediaKind};
use crate::template::render_template;
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Operation { pub from: PathBuf, pub to: PathBuf, pub action: Action }
#[derive(Debug, Default)]
pub struct PlanResult { pub ops: Vec<Operation>, pub skipped_unknown: Vec<PathBuf>, pub skipped_conflicts: Vec<PathBuf> }

pub struct PlanOptions<'a> {
    pub format: &'a str,
    pub action: Action,
    pub output: Option<&'a Path>,
    pub use_tmdb: bool,
    pub tmdb_key: Option<&'a str>,
    pub language: &'a str,
    pub http_client: Option<&'a dyn HttpClient>,
}

pub fn build_plan(files: &[PathBuf], opts: &PlanOptions<'_>) -> Result<PlanResult, Box<dyn Error>> {
    let mut result = PlanResult::default();
    for file in files {
        let mut parsed = parse_media(file);
        if parsed.kind == MediaKind::Unknown { result.skipped_unknown.push(file.clone()); continue; }

        if opts.use_tmdb && parsed.kind == MediaKind::Movie {
            if let (Some(key), Some(client), Some(title)) = (opts.tmdb_key, opts.http_client, parsed.title_guess.clone()) {
                if let Some(found) = match_movie(client, key, &MovieQuery { title, year: parsed.year_guess, language: opts.language.to_string() })? {
                    parsed.title_guess = Some(found.title);
                    parsed.year_guess = found.year;
                }
            }
        }
        let rendered = render_template(opts.format, &parsed);
        let dest = opts.output.map(|p| p.to_path_buf()).or_else(|| file.parent().map(|p| p.to_path_buf())).unwrap_or_else(|| PathBuf::from("."));
        let to = dest.join(if rendered.is_empty() { "untitled" } else { &rendered }.replace('/', "_"));
        if to.exists() && to != *file { result.skipped_conflicts.push(to); continue; }
        result.ops.push(Operation { from: file.clone(), to, action: opts.action.clone() });
    }
    Ok(result)
}
