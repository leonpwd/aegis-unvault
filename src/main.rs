use crate::models::Vault;


mod models;
mod decrypt;
mod io;
mod tui;

use std::fs::File;
use std::io::Read;


fn main() {
    // File selection
    let file_path = match io::prompt_file_path() {
        Some(p) => p,
        None => {
            eprintln!("No file selected.");
            return;
        }
    };

    // Reading file
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Unable to open file.");
            return;
        }
    };
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        eprintln!("Error reading file.");
        return;
    }

    // Parsing JSON
    let vault: Vault = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid JSON file.");
            return;
        }
    };

    // Secure password input (rpassword)
    let password = match rpassword::prompt_password("Password: ") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Password not entered.");
            return;
        }
    };

    // Decryption
    let decrypted = match decrypt::decrypt_vault(&vault, password.as_bytes()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    // Display TUI
        use crate::tui::run_tui;
        if let Err(e) = run_tui(&decrypted, &file_path) {
        eprintln!("TUI error: {}", e);
    }
}

