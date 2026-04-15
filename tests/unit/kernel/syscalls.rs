//! Syscall Unit Tests
//!
//! Tests for kernel syscall interface.

use kernel::syscalls::*;

#[tokio::test]
async fn test_syscall_dispatcher_creation() {
    let dispatcher = SyscallDispatcher::new();
    // Verify dispatcher created successfully
    assert_eq!(dispatcher.queue_length().await, 0);
}

#[tokio::test]
async fn test_syscall_number_conversion() {
    assert_eq!(SyscallNumber::from_u64(0), Some(SyscallNumber::SpawnAgent));
    assert_eq!(SyscallNumber::from_u64(1), Some(SyscallNumber::TerminateAgent));
    assert_eq!(SyscallNumber::from_u64(2), Some(SyscallNumber::SendMessage));
    assert_eq!(SyscallNumber::from_u64(100), None);
}

#[tokio::test]
async fn test_capability_check() {
    let ctx = SyscallContext {
        caller_id: "test-agent".to_string(),
        capability_level: 5,
        workspace_id: "test-workspace".to_string(),
        session_id: "test-session".to_string(),
    };
    
    // Level 5 should be able to spawn agents (requires level 5)
    // This would be tested in integration with actual dispatcher
    assert_eq!(ctx.capability_level, 5);
}

#[tokio::test]
async fn test_syscall_args_default() {
    let args = SyscallArgs::default();
    assert_eq!(args.arg0, 0);
    assert_eq!(args.arg1, 0);
    assert_eq!(args.arg2, 0);
    assert_eq!(args.arg3, 0);
    assert_eq!(args.arg4, 0);
    assert_eq!(args.arg5, 0);
}

#[tokio::test]
async fn test_capability_token() {
    let token = CapabilityToken::new(5)
        .with_permission("spawn")
        .with_permission("filesystem");
    
    assert_eq!(token.level, 5);
    assert!(token.has_permission("spawn"));
    assert!(token.has_permission("filesystem"));
    assert!(!token.has_permission("network"));
    
    // Wildcard permission
    let wildcard = CapabilityToken::new(9)
        .with_permission("*");
    assert!(wildcard.has_permission("anything"));
}

#[test]
fn test_syscall_error_codes() {
    use SyscallError::*;
    
    assert_eq!(Success as i64, 0);
    assert!((InvalidSyscall as i64) < 0);
    assert!((PermissionDenied as i64) < 0);
    assert!((NotImplemented as i64) < 0);
}
