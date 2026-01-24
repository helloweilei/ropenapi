use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Generate TypeScript services from OpenAPI/Swagger JSON",
    long_about = None
)]
pub struct Args {
    /// Path to swagger/openapi JSON file
    #[arg(short, long)]
    pub swagger: String,

    /// Output directory (default: current dir)
    #[arg(short, long)]
    pub out: Option<String>,

    /// Optional comma-separated tags to generate (services). If omitted, all tags are generated.
    #[arg(short, long)]
    pub tags: Option<String>,
}

pub fn parse_args() -> Args {
    Args::parse()
}
