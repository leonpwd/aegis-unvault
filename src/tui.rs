pub fn run_tui(decrypted: &str, file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::panic;
    use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
    use crossterm::event::{Event, KeyCode, KeyEvent};
    use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
    use crossterm::execute;
    use ratatui::{backend::CrosstermBackend, Terminal};
    use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
    use ratatui::layout::Rect;
    use std::io::stdout;

    // --- SANITIZE INPUT ---
    let lines: Vec<String> = decrypted
        .lines()
        .map(|line| {
            // Remove control characters except newline & tab
            let cleaned: String = line
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                .collect();

            if cleaned.contains("\"icon\"")
                || cleaned.contains("\"icon_mime\"")
                || cleaned.contains("\"icon_hash\"")
            {
                "[icon removed]".to_string()
            } else {
                cleaned
            }
        })
        .collect();

    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, crossterm::terminal::EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut scroll: usize = 0;
    let mut status = String::new();
    let mut search_mode = false;
    let mut search_query = String::new();
    let mut search_results: Vec<usize> = Vec::new();
    let mut search_index: usize = 0;

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        loop {
            terminal.draw(|f| {
                let size = f.size();
                let height = size.height as usize;
                let content_height = height.saturating_sub(3);

                let visible_lines = if scroll + content_height <= lines.len() {
                    &lines[scroll..scroll + content_height]
                } else {
                    &lines[scroll..]
                };

                let text = if !search_results.is_empty() {
                    visible_lines.iter().enumerate().map(|(i, line)| {
                        let line_index = scroll + i;
                        if search_results.contains(&line_index) {
                            format!("> {}", line)
                        } else {
                            line.clone()
                        }
                    }).collect::<Vec<String>>().join("\n")
                } else {
                    visible_lines.join("\n")
                };

                let block = Block::default()
                    .title("Contenu déchiffré")
                    .borders(Borders::ALL);

                let paragraph = Paragraph::new(text)
                    .block(block)
                    .wrap(Wrap { trim: false });

                let area = Rect {
                    x: size.x,
                    y: size.y,
                    width: size.width,
                    height: content_height as u16,
                };
                f.render_widget(paragraph, area);

                // Help bar
                let help = Paragraph::new(
                    "[e] Exporter  [q] Quitter  [↑/↓] Scroll  [/] Recherche  [n/N] Suivant/Précédent"
                );
                f.render_widget(help, Rect {
                    x: size.x,
                    y: size.y + content_height as u16,
                    width: size.width,
                    height: 1,
                });

                // Status bar
                let status_bar = Paragraph::new(status.clone());
                f.render_widget(status_bar, Rect {
                    x: size.x,
                    y: size.y + content_height as u16 + 1,
                    width: size.width,
                    height: 1,
                });

                // Search bar
                if search_mode {
                    let search_bar = Paragraph::new(format!("Recherche : {}", search_query));
                    f.render_widget(search_bar, Rect {
                        x: size.x,
                        y: size.y + content_height as u16 + 2,
                        width: size.width,
                        height: 1,
                    });
                }
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(200))? {
                match crossterm::event::read()? {
                    Event::Key(KeyEvent { code, .. }) => {
                        if search_mode {
                            match code {
                                KeyCode::Esc => {
                                    search_mode = false;
                                    search_query.clear();
                                    search_results.clear();
                                    search_index = 0;
                                }
                                KeyCode::Char(c) => search_query.push(c),
                                KeyCode::Backspace => { search_query.pop(); }
                                KeyCode::Enter => {
                                    search_results = lines.iter().enumerate()
                                        .filter(|(_, line)| line.contains(&search_query))
                                        .map(|(i, _)| i)
                                        .collect();

                                    if !search_results.is_empty() {
                                        scroll = search_results[0];
                                        search_index = 0;
                                        status = format!("{} résultats trouvés", search_results.len());
                                    } else {
                                        status = "Aucun résultat".to_string();
                                    }

                                    search_mode = false;
                                }
                                _ => {}
                            }
                        } else {
                            match code {
                                KeyCode::Char('q') | KeyCode::Esc => break,
                                KeyCode::Up => if scroll > 0 { scroll -= 1; },
                                KeyCode::Down => {
                                    let size = terminal.size()?;
                                    let content_height = size.height.saturating_sub(3) as usize;
                                    if scroll + content_height < lines.len() { scroll += 1; }
                                }
                                KeyCode::Char('e') => {
                                    use rfd::FileDialog;
                                    use std::fs;

                                    let mut default_name = String::new();
                                    if let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) {
                                        default_name.push_str(stem);
                                        default_name.push_str("_uncyphered");
                                        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                                            default_name.push('.');
                                            default_name.push_str(ext);
                                        }
                                    }

                                    if let Some(path) = FileDialog::new()
                                        .set_title("Enregistrer le fichier déchiffré")
                                        .set_file_name(&default_name)
                                        .save_file()
                                    {
                                        match fs::write(&path, decrypted) {
                                            Ok(_) => status = format!("Exporté vers {}", path.display()),
                                            Err(e) => status = format!("Erreur export : {}", e),
                                        }
                                    } else {
                                        status = "Export annulé".to_string();
                                    }
                                }
                                KeyCode::Char('/') => {
                                    search_mode = true;
                                    search_query.clear();
                                    status.clear();
                                }
                                KeyCode::Char('n') => {
                                    if !search_results.is_empty() {
                                        search_index = (search_index + 1) % search_results.len();
                                        scroll = search_results[search_index];
                                    }
                                }
                                KeyCode::Char('N') => {
                                    if !search_results.is_empty() {
                                        search_index = (search_index + search_results.len() - 1)
                                            % search_results.len();
                                        scroll = search_results[search_index];
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }));

    execute!(terminal.backend_mut(), DisableBracketedPaste).ok();
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Erreur interne dans le TUI (panic)".into()),
    }
}