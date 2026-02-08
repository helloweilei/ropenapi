use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author = "1060290944@qq.com",
    version = "0.0.1",
    about = "Generate TypeScript services from OpenAPI/Swagger JSON",
    long_about = None
)]
pub struct Args {
    /// Path to swagger/openapi JSON file
    #[arg(short, long)]
    pub swagger: String,

    /// Output directory (default: services/)
    #[arg(short, long)]
    pub out: Option<String>,

    /// Service path, alias for output directory
    #[arg(long)]
    pub service_path: Option<String>,

    /// Optional comma-separated tags to generate (services). If omitted, all tags are generated.
    #[arg(short, long)]
    pub tags: Option<String>,
    /// Request lib path to import in generated services, e.g., 'import { request} from @/utils/request'
    #[arg(short, long, default_value = "import { request } from '@/services/request';")]
    pub request_lib_path: Option<String>,
    /// Project name, used for service folder name
    #[arg(short, long, default_value = "project-swagger")]
    pub project_name: Option<String>,
    /// Api prefix, prefix of all api urls, eg. /api
    #[arg(short, long)]
    pub api_prefix: Option<String>,
    // Namespace, All declarations will be wrapped in this namespace
    // #[arg(short, long)]
    // pub namespace: Option<String>,
}

pub fn parse_args() -> Args {
    Args::parse()
}
