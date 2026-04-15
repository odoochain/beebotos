use beebotos_agents::security::{Sandbox, Capability};
use beebotos_kernel::security::Permission;

#[test]
fn test_capability_check() {
    // Create capability
    let cap = Capability::new("resource_1")
        .with_permission(Permission::READ)
        .with_permission(Permission::WRITE);

    // Verify permissions
    assert!(cap.has_permission(Permission::READ));
    assert!(cap.has_permission(Permission::WRITE));
    assert!(!cap.has_permission(Permission::EXECUTE));
}

#[test]
fn test_sandbox_isolation() {
    let sandbox = Sandbox::new()
        .with_memory_limit(1024 * 1024) // 1MB
        .with_cpu_limit(100) // 100ms
        .build();

    // Attempt to exceed memory limit
    let result = sandbox.execute(|| {
        // Memory-intensive operation
        let _large_vec: Vec<u8> = vec![0; 2 * 1024 * 1024]; // 2MB
    });

    assert!(result.is_err());
}

#[tokio::test]
async fn test_permission_escalation() {
    let ctx = beebotos_test_utils::TestContext::new().await;
    let agent = ctx.create_test_agent().await;

    // Agent tries to escalate privileges
    let result = agent
        .request_capability("admin://system", &[Permission::ADMIN])
        .await;

    // Should be denied
    assert!(result.is_err());
}

#[tokio::test]
async fn test_secure_communication() {
    let ctx = beebotos_test_utils::TestContext::new().await;
    let agent_a = ctx.create_test_agent().await;
    let agent_b = ctx.create_test_agent().await;

    // Establish secure channel
    let channel = agent_a
        .establish_secure_channel(&agent_b.get_address())
        .await
        .unwrap();

    // Send encrypted message
    let msg = b"secret message";
    let encrypted = channel.encrypt(msg);
    let decrypted = channel.decrypt(&encrypted).unwrap();

    assert_eq!(msg.to_vec(), decrypted);
}
