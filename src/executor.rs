use crate::cli::Action;
use crate::planner::Operation;
use std::error::Error;

pub fn execute_plan(ops: &[Operation], dry_run: bool) -> Result<(), Box<dyn Error>> {
    if dry_run { return Ok(()); }
    for op in ops {
        if let Some(parent) = op.to.parent() { std::fs::create_dir_all(parent)?; }
        match op.action {
            Action::Rename | Action::Move => std::fs::rename(&op.from, &op.to)?,
            Action::Copy => { std::fs::copy(&op.from, &op.to)?; }
        }
    }
    Ok(())
}
