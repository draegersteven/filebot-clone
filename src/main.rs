use mybot::cli::{help_text, parse_args, Commands, PlanArgs};
use mybot::executor::execute_plan;
use mybot::matcher::{match_movie, required_tmdb_key, MovieQuery, NoopHttpClient};
use mybot::parser::{parse_media, MediaKind};
use mybot::planner::{build_plan, PlanOptions};
use mybot::scanner::scan_files;
use std::error::Error;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = parse_args(&args).map_err(|e| format!("{e}\n\n{}", help_text()))?;
    match cmd {
        Commands::Help => println!("{}", help_text()),
        Commands::Scan { path, recursive } => {
            for file in scan_files(&path, recursive)? {
                let p = parse_media(&file);
                println!("{} => kind={:?} title={:?} year={:?} season={:?} episode={:?}", file.display(), p.kind, p.title_guess, p.year_guess, p.season, p.episode);
            }
        }
        Commands::Plan(args) => handle_plan_like(args, false)?,
        Commands::Apply(args) => handle_plan_like(args, true)?,
        Commands::MatchMovie { path, tmdb_key, language, recursive, .. } => {
            let key = required_tmdb_key(&tmdb_key)?;
            let client = NoopHttpClient;
            for file in scan_files(&path, recursive)? {
                let parsed = parse_media(&file);
                if parsed.kind != MediaKind::Movie { println!("SKIP {} (not a movie)", file.display()); continue; }
                if let Some(title) = parsed.title_guess {
                    let query = MovieQuery { title, year: parsed.year_guess, language: language.clone() };
                    match match_movie(&client, &key, &query)? {
                        Some(m) => println!("{} => {} ({:?}) score={:.2}", file.display(), m.title, m.year, m.score),
                        None => println!("{} => no result", file.display()),
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_plan_like(args: PlanArgs, apply: bool) -> Result<(), Box<dyn Error>> {
    let files = scan_files(&args.path, args.recursive)?;
    let use_tmdb = args.db.as_deref() == Some("tmdb");
    let key = if use_tmdb { Some(required_tmdb_key(&args.tmdb_key)?) } else { None };
    let client = NoopHttpClient;
    let plan = build_plan(&files, &PlanOptions {
        format: &args.format, action: args.action, output: args.output.as_deref(),
        use_tmdb, tmdb_key: key.as_deref(), language: "de-DE", http_client: Some(&client),
    })?;
    for op in &plan.ops { println!("PLAN {:?}: {} -> {}", op.action, op.from.display(), op.to.display()); }
    for u in &plan.skipped_unknown { println!("SKIP unknown: {}", u.display()); }
    for c in &plan.skipped_conflicts { println!("SKIP conflict: {}", c.display()); }
    if apply { execute_plan(&plan.ops, args.dry_run)?; }
    Ok(())
}
