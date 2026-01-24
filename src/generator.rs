use anyhow::{ Context, Result };
use std::fs;
use std::path::Path;

use crate::models::Service;

/// Write all services to disk
pub fn write_services(out_dir: &Path, services: &[Service]) -> Result<()> {
    let services_dir = out_dir.join("services");
    ensure_dir(&services_dir)?;

    for service in services {
        write_service(&services_dir, service)?;
    }

    Ok(())
}

/// Write a single service (both api file and types file)
fn write_service(services_root: &Path, service: &Service) -> Result<()> {
    let service_dir = services_root.join(&service.name);
    ensure_dir(&service_dir)?;

    let api_file_path = service_dir.join(format!("{}-swagger.ts", service.name));
    let types_file_path = service_dir.join("types.ts");

    write_api_file(&api_file_path, service)?;
    write_types_file(&types_file_path, service)?;

    println!(
        "  ✓ Generated {}/{}",
        service.name,
        api_file_path.file_name().unwrap().to_string_lossy()
    );
    println!(
        "  ✓ Generated {}/{}",
        service.name,
        types_file_path.file_name().unwrap().to_string_lossy()
    );

    Ok(())
}

/// Write API service file
fn write_api_file(path: &Path, service: &Service) -> Result<()> {
    let mut content = String::new();

    // Add header with imports
    content.push_str("import request from '@/utils/http';\n");
    content.push_str("import * as Types from './types';\n");
    content.push_str("import type { IResponse } from '@/types';\n\n");

    // Add operations with proper spacing
    for (idx, operation) in service.operations.iter().enumerate() {
        content.push_str(&operation.to_typescript_function());
        if idx < service.operations.len() - 1 {
            content.push_str("\n\n");
        }
    }

    content.push('\n');

    fs
        ::write(path, &content)
        .with_context(|| format!("Failed to write API file: {}", path.display()))?;

    Ok(())
}

/// Write types definition file
fn write_types_file(path: &Path, service: &Service) -> Result<()> {
    let mut content = String::new();

    content.push_str("/**\n");
    content.push_str(&format!(" * Type definitions for {} service\n", service.name));
    content.push_str(" */\n\n");

    for type_def in service.type_definitions.values() {
        content.push_str(&type_def.to_typescript());
        content.push_str("\n\n");
    }

    // Add placeholder types for operations if not in definitions
    for operation in &service.operations {
        if
            !type_already_exists(&content, &operation.request_type) &&
            operation.request_type != "any"
        {
            content.push_str(&format!("export type {} = any;\n\n", operation.request_type));
        }
        if
            !type_already_exists(&content, &operation.response_type) &&
            operation.response_type != "any"
        {
            content.push_str(&format!("export type {} = any;\n\n", operation.response_type));
        }
    }

    // Trim trailing whitespace
    let content = content.trim_end().to_string() + "\n";

    fs
        ::write(path, &content)
        .with_context(|| format!("Failed to write types file: {}", path.display()))?;

    Ok(())
}

/// Check if type is already defined in content
fn type_already_exists(content: &str, type_name: &str) -> bool {
    content.contains(&format!("export type {} =", type_name))
}

/// Ensure directory exists
fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs
            ::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}
