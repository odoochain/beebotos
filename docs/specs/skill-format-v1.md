# Skill Format Specification v1.0

## Overview

This document defines the standard format for BeeBotOS skills - reusable capabilities that can be installed on agents.

## Skill Structure

A skill is a package containing:

```
skill/
├── skill.yaml          # Skill manifest
├── README.md           # Documentation
├── src/                # Source code
│   └── lib.rs          # Entry point (Rust)
│   └── skill.py        # Entry point (Python)
│   └── index.ts        # Entry point (TypeScript)
├── tests/              # Test files
├── schemas/            # JSON schemas
│   ├── input.json
│   └── output.json
└── metadata.json       # Generated metadata
```

## Manifest Format (skill.yaml)

```yaml
# Required fields
name: my-skill
version: 1.0.0
description: A brief description
author: Author Name <email@example.com>
license: MIT

# Categorization
category: coding
tags: [rust, code-generation]

# Runtime configuration
runtime:
  type: wasm          # wasm, python, node
  entrypoint: lib.wasm
  
# Resource limits
resources:
  memory: 128MB
  timeout: 30s
  cpu: 100ms

# Schema definitions
schema:
  input:
    type: object
    required: [code]
    properties:
      code:
        type: string
        description: Code to analyze
      language:
        type: string
        enum: [rust, python, solidity]
        
  output:
    type: object
    properties:
      suggestions:
        type: array
        items:
          type: object
          properties:
            line:
              type: integer
            message:
              type: string

# Configuration options
config:
  strict_mode:
    type: boolean
    default: false
    description: Enable strict checking
  max_suggestions:
    type: integer
    default: 10
    
# Dependencies (for non-WASM skills)
dependencies:
  python:
    - numpy>=1.24
    - pandas>=2.0
  node:
    - lodash
```

## Runtime Types

### WebAssembly (WASM)

```rust
// lib.rs
use beebotos_sdk::{skill, Context, Result};
use serde_json::Value;

#[skill(name = "my-skill")]
pub async fn handle(ctx: Context, input: Value) -> Result<Value> {
    // Implementation
    Ok(result)
}
```

**Requirements:**
- Target: `wasm32-wasi`
- Interface: WASI Preview 1
- Maximum size: 10MB

### Python

```python
# skill.py
from beebotos_sdk import skill, Context

@skill(name="my-skill")
async def handle(ctx: Context, input: dict) -> dict:
    # Implementation
    return result
```

**Requirements:**
- Python 3.10+
- Async functions only
- Sandboxed execution

### TypeScript/Node.js

```typescript
// index.ts
import { skill, Context } from '@beebotos/sdk';

@skill({ name: 'my-skill' })
export async function handle(ctx: Context, input: any): Promise<any> {
  // Implementation
  return result;
}
```

**Requirements:**
- Node.js 18+
- ES modules
- Sandboxed execution

## Schema Definition

Input/output schemas use JSON Schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "text": {
      "type": "string",
      "description": "Input text to process",
      "maxLength": 10000
    }
  },
  "required": ["text"]
}
```

## Versioning

Follow semantic versioning:

- **MAJOR**: Breaking changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes

## Security Requirements

1. **Code Signing**: All skills must be signed
2. **Sandboxing**: Network access restricted by default
3. **Resource Limits**: Enforced by runtime
4. **Audit**: Code review for marketplace skills

## Publishing

### To Registry

```bash
beebotos skill publish ./my-skill --registry default
```

### Verification

Skills are verified for:
- Schema compliance
- Resource usage
- Security scan
- Functionality test

## Examples

See `/examples/skill_template/` for complete examples in each language.
