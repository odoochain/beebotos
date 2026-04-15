#!/usr/bin/env python3
"""Code generation utilities for BeeBotOS"""

import json
import re
from pathlib import Path
from typing import Dict, List, Optional


def generate_syscall_enum(syscalls: List[Dict]) -> str:
    """Generate syscall enum from definition"""
    lines = ["#[repr(u64)]", "#[derive(Debug, Clone, Copy)]", "pub enum SyscallNumber {"]
    
    for syscall in syscalls:
        name = syscall["name"]
        number = syscall["number"]
        lines.append(f"    {name} = {number},")
    
    lines.append("}")
    return "\n".join(lines)


def generate_capability_checks(caps: List[Dict]) -> str:
    """Generate capability check functions"""
    lines = ["impl CapabilityChecker {"]
    
    for cap in caps:
        name = cap["name"]
        level = cap["level"]
        lines.append(f"    /// Check {name}")
        lines.append(f"    pub fn check_{name.to_snake()}(&self, agent_id: &AgentId) -> Result<()> {{")
        lines.append(f"        self.require_level(agent_id, CapabilityLevel::{level})")
        lines.append(f"    }}")
        lines.append("")
    
    lines.append("}")
    return "\n".join(lines)


def generate_proto_messages(messages: List[Dict]) -> str:
    """Generate protobuf message definitions"""
    lines = ["syntax = \"proto3\";", ""]
    
    for msg in messages:
        name = msg["name"]
        fields = msg.get("fields", [])
        
        lines.append(f"message {name} {{")
        for i, field in enumerate(fields, 1):
            fname = field["name"]
            ftype = field["type"]
            lines.append(f"    {ftype} {fname} = {i};")
        lines.append("}")
        lines.append("")
    
    return "\n".join(lines)


def generate_openapi_spec(routes: List[Dict]) -> Dict:
    """Generate OpenAPI specification"""
    spec = {
        "openapi": "3.0.0",
        "info": {
            "title": "BeeBotOS API",
            "version": "1.0.0",
        },
        "paths": {},
    }
    
    for route in routes:
        path = route["path"]
        method = route["method"].lower()
        
        if path not in spec["paths"]:
            spec["paths"][path] = {}
        
        spec["paths"][path][method] = {
            "summary": route.get("summary", ""),
            "parameters": route.get("parameters", []),
            "responses": {
                "200": {
                    "description": "Success",
                    "content": {
                        "application/json": {
                            "schema": route.get("response_schema", {})
                        }
                    }
                }
            }
        }
    
    return spec


def generate_dao_abi(contract: Dict) -> str:
    """Generate DAO contract ABI"""
    abi = []
    
    for func in contract.get("functions", []):
        abi.append({
            "type": "function",
            "name": func["name"],
            "inputs": func.get("inputs", []),
            "outputs": func.get("outputs", []),
            "stateMutability": func.get("mutability", "view"),
        })
    
    for event in contract.get("events", []):
        abi.append({
            "type": "event",
            "name": event["name"],
            "inputs": event.get("inputs", []),
            "anonymous": event.get("anonymous", False),
        })
    
    return json.dumps(abi, indent=2)


class StringConverter:
    """String case converters"""
    
    @staticmethod
    def to_snake(s: str) -> str:
        return re.sub(r'(?<!^)(?=[A-Z])', '_', s).lower()
    
    @staticmethod
    def to_camel(s: str) -> str:
        components = s.split('_')
        return components[0] + ''.join(x.title() for x in components[1:])
    
    @staticmethod
    def to_pascal(s: str) -> str:
        return ''.join(x.title() for x in s.split('_'))


def main():
    """CLI entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description="BeeBotOS code generator")
    parser.add_argument("input", help="Input JSON file")
    parser.add_argument("--type", required=True, choices=["syscall", "capability", "proto", "openapi", "abi"])
    parser.add_argument("--output", required=True, help="Output file")
    
    args = parser.parse_args()
    
    with open(args.input) as f:
        data = json.load(f)
    
    generators = {
        "syscall": generate_syscall_enum,
        "capability": generate_capability_checks,
        "proto": generate_proto_messages,
        "abi": generate_dao_abi,
    }
    
    if args.type == "openapi":
        result = generate_openapi_spec(data)
        output = json.dumps(result, indent=2)
    else:
        output = generators[args.type](data)
    
    Path(args.output).write_text(output)
    print(f"Generated: {args.output}")


if __name__ == "__main__":
    main()
