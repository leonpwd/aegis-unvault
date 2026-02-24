use crate::models::Vault;
use base64::Engine;
use base64::engine::general_purpose;
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use scrypt::{scrypt, Params as ScryptParams};

pub fn decrypt_vault(vault: &Vault, password: &[u8]) -> Result<String, String> {
    let slots: Vec<&crate::models::Slot> = vault.header.slots.iter().filter(|s| s.slot_type == 1).collect();
    let mut master_key: Option<Vec<u8>> = None;
    for slot in slots.iter() {
        let salt = match hex::decode(&slot.salt) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Salt hex invalide: {}", e);
                continue;
            }
        };
        let log_n = (slot.n as f64).log2() as u8;
        let params = match ScryptParams::new(log_n, slot.r, slot.p, 32) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Paramètres Scrypt invalides: {:?}", e);
                continue;
            }
        };
        let mut key = [0u8; 32];
        match scrypt(password, &salt, &params, &mut key) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Erreur Scrypt: {:?}", e);
                continue;
            }
        }
        let nonce_vec = match hex::decode(&slot.key_params.nonce) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Nonce hex invalide: {}", e);
                continue;
            }
        };
        let nonce: [u8; 12] = match nonce_vec.try_into() {
            Ok(arr) => arr,
            Err(_) => {
                eprintln!("Nonce hex invalide (taille)");
                continue;
            }
        };
        let mut ciphertext = match hex::decode(&slot.key) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Key hex invalide: {}", e);
                continue;
            }
        };
        let tag = match hex::decode(&slot.key_params.tag) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Tag hex invalide: {}", e);
                continue;
            }
        };
        ciphertext.extend_from_slice(&tag);
        let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(&key));
        match cipher.decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref()) {
            Ok(mk) => {
                master_key = Some(mk);
                break;
            },
            Err(e) => {
                eprintln!("Erreur de déchiffrement master_key: {:?}", e);
                continue;
            },
        }
    }
    let master_key = master_key.ok_or("Impossible de déchiffrer la master key avec ce mot de passe")?;
    let content = general_purpose::STANDARD.decode(&vault.db).map_err(|_| "Base64 invalide")?;
    let params = &vault.header.params;
    let nonce_vec = match hex::decode(&params.nonce) {
        Ok(v) => v,
        Err(_) => return Err("Nonce hex invalide".to_string()),
    };
    let nonce: [u8; 12] = match nonce_vec.try_into() {
        Ok(arr) => arr,
        Err(_) => return Err("Taille de nonce incorrecte".to_string()),
    };
    let mut ciphertext = content;
    let tag = hex::decode(&params.tag).map_err(|_| "Tag hex invalide")?;
    ciphertext.extend_from_slice(&tag);
    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(&master_key));
    let db = cipher.decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref()).map_err(|_| "Erreur de déchiffrement du vault")?;
    Ok(String::from_utf8(db).map_err(|_| "UTF-8 invalide")?)
}
