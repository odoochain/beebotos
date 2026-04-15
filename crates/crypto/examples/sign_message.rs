//! Sign and verify message example

use beebotos_crypto::signatures::{Ed25519Signer, SignerTrait};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Cryptographic Signing Example\n");

    // Generate keypair
    println!("1. Generating Ed25519 keypair:");
    let signer = Ed25519Signer::new();
    println!("   ✅ Keypair generated");
    println!("   Public key: {:?}", signer.public_key());

    // Sign message
    println!("\n2. Signing message:");
    let message = b"Hello, BeeBotOS!";
    println!("   Message: {}", String::from_utf8_lossy(message));

    let signature = signer.sign(message)?;
    println!("   ✅ Message signed");
    println!(
        "   Signature ({} bytes): {:?}",
        signature.len(),
        &signature[..8.min(signature.len())]
    );

    // Verify signature
    println!("\n3. Verifying signature:");
    use beebotos_crypto::signatures::VerifierTrait;
    let is_valid = signer.verify(message, &signature)?;
    println!("   ✅ Signature valid: {}", is_valid);

    println!("\n✅ Example complete!");
    Ok(())
}
