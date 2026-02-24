use std::path::PathBuf;

pub fn prompt_file_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Aegis", &["aegis", "json", "txt"])
        .set_title("Sélectionnez un fichier Aegis")
        .pick_file()
}

