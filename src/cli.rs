use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Rename,
    Move,
    Copy,
}

#[derive(Debug)]
pub enum Commands {
    Scan { path: PathBuf, recursive: bool },
    Plan(PlanArgs),
    Apply(PlanArgs),
    MatchMovie {
        path: PathBuf,
        tmdb_key: Option<String>,
        language: String,
        dry_run: bool,
        recursive: bool,
    },
    Help,
}

#[derive(Debug)]
pub struct PlanArgs {
    pub path: PathBuf,
    pub recursive: bool,
    pub format: String,
    pub action: Action,
    pub output: Option<PathBuf>,
    pub dry_run: bool,
    pub db: Option<String>,
    pub tmdb_key: Option<String>,
}

pub fn parse_args(args: &[String]) -> Result<Commands, String> {
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        return Ok(Commands::Help);
    }
    match args[1].as_str() {
        "scan" => {
            if args.len() < 3 { return Err("scan requires <path>".into()); }
            Ok(Commands::Scan { path: args[2].clone().into(), recursive: has_flag(args, "--recursive") })
        }
        "plan" | "apply" => {
            if args.len() < 3 { return Err("plan/apply requires <path>".into()); }
            let format = value_of(args, "--format").ok_or("--format required")?;
            let action = match value_of(args, "--action").as_deref().unwrap_or("rename") {
                "rename" => Action::Rename,
                "move" => Action::Move,
                "copy" => Action::Copy,
                _ => return Err("invalid --action".into()),
            };
            let cmd = PlanArgs {
                path: args[2].clone().into(),
                recursive: has_flag(args, "--recursive"),
                format,
                action,
                output: value_of(args, "--output").map(PathBuf::from),
                dry_run: has_flag(args, "--dry-run"),
                db: value_of(args, "--db"),
                tmdb_key: value_of(args, "--tmdb-key"),
            };
            if args[1] == "plan" { Ok(Commands::Plan(cmd)) } else { Ok(Commands::Apply(cmd)) }
        }
        "match" => {
            if args.get(2).map(|s| s.as_str()) != Some("movie") { return Err("only 'match movie' supported".into()); }
            if args.len() < 4 { return Err("match movie requires <path>".into()); }
            Ok(Commands::MatchMovie {
                path: args[3].clone().into(),
                tmdb_key: value_of(args, "--tmdb-key"),
                language: value_of(args, "--language").unwrap_or_else(|| "de-DE".into()),
                dry_run: has_flag(args, "--dry-run"),
                recursive: has_flag(args, "--recursive"),
            })
        }
        _ => Err("unknown subcommand".into()),
    }
}

fn has_flag(args: &[String], name: &str) -> bool { args.iter().any(|a| a == name) }
fn value_of(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone())
}

pub fn help_text() -> &'static str {
    "mybot usage:\n  mybot scan <path> [--recursive]\n  mybot plan <path> [--recursive] --format \"<template>\" [--action rename|move|copy] [--output <dir>] [--dry-run] [--db tmdb --tmdb-key <key>]\n  mybot apply <path> [--recursive] --format \"<template>\" [--action rename|move|copy] [--output <dir>] [--dry-run] [--db tmdb --tmdb-key <key>]\n  mybot match movie <path> --tmdb-key <key> [--language de-DE] [--dry-run]"
}
