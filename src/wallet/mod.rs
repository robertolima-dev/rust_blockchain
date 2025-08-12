use rand::rngs::OsRng;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey, ecdsa::Signature};

/// Generate a new secp256k1 keypair and return (priv_hex, pub_hex_compressed, address_hex).
/// Address is simply the hex of the compressed public key (didactic).
pub fn generate_keypair_hex() -> (String, String, String) {
    let secp = Secp256k1::new(); // context with All capabilities
    let (sk, pk) = secp.generate_keypair(&mut OsRng);
    let sk_hex = hex::encode(sk.secret_bytes());
    let pk_hex = hex::encode(pk.serialize()); // compressed (33 bytes)
    let address = pk_hex.clone();
    (sk_hex, pk_hex, address)
}

/// Derive address (hex of compressed pubkey) from a given hex pubkey.
/// Returns normalized hex (lowercase) if valid.
pub fn pubkey_to_address_hex(pubkey_hex: &str) -> Result<String, &'static str> {
    let bytes = hex::decode(pubkey_hex).map_err(|_| "invalid pubkey hex")?;
    let pk = PublicKey::from_slice(&bytes).map_err(|_| "invalid pubkey bytes")?;
    Ok(hex::encode(pk.serialize()))
}

/// Verify a signature (hex DER) against the given pubkey (hex, compressed) and message hash (32 bytes).
pub fn verify_signature_hex(
    pubkey_hex: &str,
    sig_hex: &str,
    msg32: [u8; 32],
) -> Result<bool, &'static str> {
    // Use verification-only context (correct API for secp256k1 0.28)
    let secp = Secp256k1::verification_only();

    let sig_bytes = hex::decode(sig_hex).map_err(|_| "invalid signature hex")?;
    let sig = Signature::from_der(&sig_bytes).map_err(|_| "invalid DER signature")?;

    let pk_bytes = hex::decode(pubkey_hex).map_err(|_| "invalid pubkey hex")?;
    let pk = PublicKey::from_slice(&pk_bytes).map_err(|_| "invalid pubkey bytes")?;

    let msg = Message::from_slice(&msg32).map_err(|_| "invalid message length")?;
    Ok(secp.verify_ecdsa(&msg, &sig, &pk).is_ok())
}
