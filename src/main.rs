use crate::models::Vault;


mod models;
mod decrypt;
mod io;
mod tui;

use std::fs::File;
use std::io::Read;


fn main() {
    // Sélection du fichier
    let file_path = match io::prompt_file_path() {
        Some(p) => p,
        None => {
            eprintln!("Aucun fichier sélectionné.");
            return;
        }
    };

    // Lecture du fichier
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Impossible d'ouvrir le fichier.");
            return;
        }
    };
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        eprintln!("Erreur de lecture du fichier.");
        return;
    }

    // Parsing JSON
    let vault: Vault = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Fichier JSON invalide.");
            return;
        }
    };

    // Saisie du mot de passe sécurisée (rpassword)
    let password = match rpassword::prompt_password("Mot de passe : ") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Mot de passe non saisi.");
            return;
        }
    };

    // Déchiffrement
    let decrypted = match decrypt::decrypt_vault(&vault, password.as_bytes()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Erreur : {}", e);
            return;
        }
    };

    // Affichage TUI
        use crate::tui::run_tui;
        if let Err(e) = run_tui(&decrypted, &file_path) {
        eprintln!("Erreur TUI : {}", e);
    }
}

