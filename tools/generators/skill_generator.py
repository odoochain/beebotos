#!/usr/bin/env python3
"""
BeeBotOS Skill Generator
Generates skill templates for BeeBotOS agents
"""

import argparse
import json
import os
from datetime import datetime
from pathlib import Path

SKILL_TEMPLATES = {
    "rust": {
        "files": {
            "Cargo.toml": """[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
beebotos-sdk = {{ path = "../../../crates/sdk" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"

[lib]
crate-type = ["cdylib", "rlib"]
""",
            "src/lib.rs": """use beebotos_sdk::{{skill, Context, Result}};
use serde_json::Value;

/// Skill metadata
pub const SKILL_NAME: &str = "{name}";
pub const SKILL_VERSION: &str = "0.1.0";
pub const SKILL_DESCRIPTION: &str = "{description}";

/// Main skill handler
#[skill(
    name = "{name}",
    description = "{description}",
    version = "0.1.0"
)]
pub async fn handle(ctx: Context, input: Value) -> Result<Value> {{
    // TODO: Implement skill logic
    
    let result = serde_json::json!({{
        "status": "success",
        "message": "Skill executed successfully"
    }});
    
    Ok(result)
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[tokio::test]
    async fn test_skill() {{
        let ctx = Context::new();
        let input = serde_json::json!({{"test": "data"}});
        let result = handle(ctx, input).await;
        assert!(result.is_ok());
    }}
}}
""",
            "skill.yaml": """name: {name}
version: 0.1.0
description: {description}
author: {author}
category: {category}

entrypoint:
  type: wasm
  module: target/wasm32-wasi/release/{name}.wasm

schema:
  input:
    type: object
    properties:
      # Define input schema here
  output:
    type: object
    properties:
      # Define output schema here

resources:
  memory: 128MB
  timeout: 30s
""",
            "README.md": """# {name}

{description}

## Usage

```rust
use beebotos_sdk::Agent;

let agent = Agent::new();
agent.install_skill("{name}").await?;
```

## Configuration

Configure the skill in your agent configuration:

```yaml
skills:
  - name: {name}
    config:
      # Add configuration here
```

## Development

Build the skill:
```bash
cargo build --target wasm32-wasi --release
```

Test the skill:
```bash
cargo test
```
"""
        }
    },
    
    "python": {
        "files": {
            "skill.py": """\"\"\"
{name} - {description}
\"\"\"

from beebotos_sdk import skill, Context
from typing import Dict, Any

@skill(
    name="{name}",
    description="{description}",
    version="0.1.0"
)
async def handle(ctx: Context, input: Dict[str, Any]) -> Dict[str, Any]:
    \"\"\"
    Main skill handler
    \"\"\"
    # TODO: Implement skill logic
    
    return {{
        "status": "success",
        "message": "Skill executed successfully"
    }}
""",
            "skill.yaml": """name: {name}
version: 0.1.0
description: {description}
author: {author}
category: {category}

entrypoint:
  type: python
  module: skill.py
  handler: handle

schema:
  input:
    type: object
  output:
    type: object

resources:
  memory: 256MB
  timeout: 60s
""",
            "requirements.txt": """beebotos-sdk>=0.1.0
""",
            "README.md": """# {name}

{description}

## Installation

```bash
pip install -r requirements.txt
```

## Usage

See [skill.yaml](skill.yaml) for configuration options.
"""
        }
    },
    
    "typescript": {
        "files": {
            "package.json": """{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "{description}",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {{
    "build": "tsc",
    "test": "jest"
  }},
  "dependencies": {{
    "@beebotos/sdk": "^0.1.0"
  }},
  "devDependencies": {{
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0"
  }}
}}
""",
            "src/index.ts": """import {{ skill, Context }} from '@beebotos/sdk';

interface Input {{
    // Define input type
}}

interface Output {{
    status: string;
    message: string;
}}

export const SKILL_NAME = '{name}';
export const SKILL_VERSION = '0.1.0';

@skill({{
    name: '{name}',
    description: '{description}',
    version: '0.1.0'
}})
export async function handle(ctx: Context, input: Input): Promise<Output> {{
    // TODO: Implement skill logic
    
    return {{
        status: 'success',
        message: 'Skill executed successfully'
    }};
}}
""",
            "tsconfig.json": """{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "commonjs",
    "lib": ["ES2022"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "experimentalDecorators": true,
    "emitDecoratorMetadata": true
  }},
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}}
""",
            "skill.yaml": """name: {name}
version: 0.1.0
description: {description}
author: {author}
category: {category}

entrypoint:
  type: node
  module: dist/index.js
  handler: handle

schema:
  input:
    type: object
  output:
    type: object
""",
            "README.md": """# {name}

{description}

## Development

```bash
npm install
npm run build
```
"""
        }
    }
}

def generate_skill(name: str, template: str, description: str, author: str, category: str, output_dir: str):
    """Generate a skill from template"""
    
    if template not in SKILL_TEMPLATES:
        raise ValueError(f"Unknown template: {template}. Available: {list(SKILL_TEMPLATES.keys())}")
    
    tmpl = SKILL_TEMPLATES[template]
    output_path = Path(output_dir) / name
    output_path.mkdir(parents=True, exist_ok=True)
    
    # Format template variables
    format_args = {
        "name": name,
        "description": description,
        "author": author,
        "category": category,
        "date": datetime.now().isoformat()
    }
    
    # Generate files
    for filename, content in tmpl["files"].items():
        file_path = output_path / filename
        file_path.parent.mkdir(parents=True, exist_ok=True)
        
        formatted_content = content.format(**format_args)
        file_path.write_text(formatted_content)
        print(f"Created: {file_path}")
    
    # Create metadata file
    metadata = {
        "name": name,
        "version": "0.1.0",
        "description": description,
        "author": author,
        "category": category,
        "template": template,
        "created": datetime.now().isoformat()
    }
    
    metadata_path = output_path / "metadata.json"
    metadata_path.write_text(json.dumps(metadata, indent=2))
    print(f"Created: {metadata_path}")
    
    print(f"\n✅ Skill '{name}' generated successfully!")
    print(f"📁 Location: {output_path.absolute()}")

def main():
    parser = argparse.ArgumentParser(description="BeeBotOS Skill Generator")
    parser.add_argument("name", help="Skill name")
    parser.add_argument("-t", "--template", choices=["rust", "python", "typescript"], 
                       default="rust", help="Template to use")
    parser.add_argument("-d", "--description", default="A BeeBotOS skill", 
                       help="Skill description")
    parser.add_argument("-a", "--author", default="Anonymous", 
                       help="Author name")
    parser.add_argument("-c", "--category", default="general", 
                       help="Skill category")
    parser.add_argument("-o", "--output", default=".", 
                       help="Output directory")
    
    args = parser.parse_args()
    
    generate_skill(
        name=args.name,
        template=args.template,
        description=args.description,
        author=args.author,
        category=args.category,
        output_dir=args.output
    )

if __name__ == "__main__":
    main()
