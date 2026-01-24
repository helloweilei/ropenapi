# ropenapi — Swagger/OpenAPI → TypeScript Services Generator

Generate TypeScript service files and type definitions from OpenAPI/Swagger JSON specifications.

## Features

- **OpenAPI 3.0 & Swagger 2.0 Support**: Automatically detects and parses both formats
- **Tag-based Organization**: Groups API operations by tag into separate service files
- **Type Extraction**: Automatically extracts type definitions from schemas
- **Flexible Filtering**: Generate specific services using `--tags` filter
- **Clean Output**: Well-organized TypeScript files with proper imports

## Building

```bash
cargo build --release
```

The binary will be in `target/release/ropenapi`.

## Usage

### Generate all services

```bash
cargo run -- --swagger path/to/swagger.json
```

### Specify output directory

```bash
cargo run -- --swagger path/to/swagger.json --out ./output
```

### Generate specific services only

```bash
cargo run -- --swagger path/to/swagger.json --tags user,order
```

### Using the built binary

```bash
./target/release/ropenapi --swagger ./api.json --out ./src --tags user
```

## Output Structure

```
services/
├── user/
│   ├── user-swagger.ts       # API functions (getUser, createUser, etc.)
│   └── types.ts              # Type definitions (User, UserResponse, etc.)
└── order/
    ├── order-swagger.ts
    └── types.ts
```

### Example: user-swagger.ts

```typescript
import request from "@/utils/http";
import * as Types from "./types";

export const getUser = async (params: Types.GetUserParams) => {
  return request<Types.GetUserParams, Types.User>({
    url: "user",
    params: params,
    method: "GET",
  });
};

export const createUser = async (data: Types.CreateUserData) => {
  return request<Types.CreateUserData, Types.User>({
    url: "user",
    data: data,
    method: "POST",
  });
};
```

### Example: types.ts

```typescript
export type GetUserParams = {
  id: number;
};

export type User = {
  id: number;
  name: string;
  email: string;
};

export type CreateUserData = {
  name: string;
  email: string;
};
```

## Architecture

The project is organized into modular components:

- **cli.rs**: Command-line argument parsing
- **parser.rs**: Swagger/OpenAPI JSON parsing and type extraction
- **models.rs**: Core data structures (Service, ApiOperation, TypeDefinition)
- **generator.rs**: File generation logic
- **main.rs**: Orchestration and entry point

## Notes

- Types referenced in operations are automatically extracted from schema definitions
- If a type cannot be resolved, it defaults to `any`
- Function names are derived from `operationId` if available, otherwise generated from method + path
- GET and DELETE requests use `params`, POST/PUT use `data`
