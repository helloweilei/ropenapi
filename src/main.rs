mod cli;
mod models;
mod parser;
mod generator;

use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = cli::parse_args();

    let out_dir = args.out.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));

    let swagger_json = parser::read_swagger_file(&args.swagger)?;

    let tag_filters: Option<HashSet<String>> = args.tags.map(|s| {
        s.split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect()
    });

    let services = parser::parse_swagger(&swagger_json, tag_filters)?;

    generator::write_services(&out_dir, &services)?;

    println!("✓ Generated services in {}/services", out_dir.display());
    Ok(())
}
