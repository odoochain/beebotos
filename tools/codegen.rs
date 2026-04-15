//! Code generation tool for BeeBotOS

use std::fs;
use std::path::Path;

fn main() {
    println!("BeeBotOS Code Generator");
    println!("========================\n");
    
    // Generate syscall numbers
    generate_syscalls().expect("Failed to generate syscalls");
    
    // Generate capability levels
    generate_capabilities().expect("Failed to generate capabilities");
    
    println!("\n✅ Code generation complete!");
}

fn generate_syscalls() -> Result<(), Box<dyn std::error::Error>> {
    let output = r#"// Auto-generated syscalls
// DO NOT EDIT MANUALLY

pub const SYSCALL_SPAWN_AGENT: u64 = 0;
pub const SYSCALL_TERMINATE_AGENT: u64 = 1;
pub const SYSCALL_SEND_MESSAGE: u64 = 2;
pub const SYSCALL_RECEIVE_MESSAGE: u64 = 3;
pub const SYSCALL_QUERY_MEMORY: u64 = 4;
pub const SYSCALL_ALLOCATE_MEMORY: u64 = 5;
pub const SYSCALL_FREE_MEMORY: u64 = 6;
pub const SYSCALL_EXECUTE_PAYMENT: u64 = 7;
pub const SYSCALL_CREATE_PROPOSAL: u64 = 8;
pub const SYSCALL_CAST_VOTE: u64 = 9;
"#;
    
    fs::write("crates/kernel/src/syscalls/generated.rs", output)?;
    println!("✅ Generated syscalls");
    Ok(())
}

fn generate_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    let output = r#"// Auto-generated capability definitions
// DO NOT EDIT MANUALLY

pub const CAP_L0_LOCAL_COMPUTE: u8 = 0;
pub const CAP_L1_FILE_READ: u8 = 1;
pub const CAP_L2_FILE_WRITE: u8 = 2;
pub const CAP_L3_NETWORK_OUT: u8 = 3;
pub const CAP_L4_NETWORK_IN: u8 = 4;
pub const CAP_L5_SPAWN_LIMITED: u8 = 5;
pub const CAP_L6_SPAWN_UNLIMITED: u8 = 6;
pub const CAP_L7_CHAIN_READ: u8 = 7;
pub const CAP_L8_CHAIN_WRITE_LOW: u8 = 8;
pub const CAP_L9_CHAIN_WRITE_HIGH: u8 = 9;
pub const CAP_L10_SYSTEM_ADMIN: u8 = 10;
"#;
    
    fs::write("crates/kernel/src/capabilities/generated.rs", output)?;
    println!("✅ Generated capabilities");
    Ok(())
}
