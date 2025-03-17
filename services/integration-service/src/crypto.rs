use crate::error::{IntegrationError, IntegrationResult};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm, Key, Nonce
};
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;

#[derive(Clone)]
pub struct CryptoService {
    key: [u8; 32],
}

impl CryptoService {
    pub fn new(key_str: &str) -> Self {
        // In production, this should be more sophisticated
        // For now, using a simple key derivation
        let mut key = [0u8; 32];
        let key_bytes = key_str.as_bytes();
        let len = std::cmp::min(key_bytes.len(), 32);
        key[..len].copy_from_slice(&key_bytes[..len]);
        
        Self { key }
    }
    
    pub fn encrypt(&self, data: &str) -> IntegrationResult<String> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        
        // Generate a random nonce
        let nonce_array = Aes256Gcm::generate_nonce(&mut rand::thread_rng());
        let nonce = Nonce::from_slice(nonce_array.as_slice());
        
        // Encrypt the data
        let ciphertext = cipher.encrypt(nonce, data.as_bytes())
            .map_err(|e| IntegrationError::Crypto(format!("Encryption failed: {}", e)))?;
        
        // Combine nonce and ciphertext and encode with base64
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);
        let encoded = general_purpose::STANDARD.encode(combined);
        
        Ok(encoded)
    }
    
    pub fn decrypt(&self, encrypted_data: &str) -> IntegrationResult<String> {
        // Decode base64
        let combined = general_purpose::STANDARD.decode(encrypted_data)
            .map_err(|e| IntegrationError::Crypto(format!("Base64 decoding failed: {}", e)))?;
        
        // Split nonce and ciphertext
        if combined.len() < 12 {
            return Err(IntegrationError::Crypto("Invalid encrypted data format".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Create cipher and decrypt
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| IntegrationError::Crypto(format!("Decryption failed: {}", e)))?;
        
        // Convert to string
        String::from_utf8(plaintext)
            .map_err(|e| IntegrationError::Crypto(format!("UTF-8 decoding failed: {}", e)))
    }
}
