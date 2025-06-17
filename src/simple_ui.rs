use crate::config::{AppConfig, GameInstallation, GameVersion};
use crate::installation::InstallationValidator;
use std::path::PathBuf;

pub struct SimpleInstallationUI {
    config: AppConfig,
    found_installations: Vec<PathBuf>,
}

impl SimpleInstallationUI {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = AppConfig::load()?;
        let found_installations = InstallationValidator::scan_common_paths();
        
        Ok(Self {
            config,
            found_installations,
        })
    }

    pub fn handle_input(&mut self) -> UIResult {
        println!("\n=== CTP Client - Game Selection ===");
        
        if self.config.get_installations().is_empty() {
            println!("No game installations found.");
            self.show_auto_detected();
        } else {
            println!("Available installations:");
            for (i, (_id, installation)) in self.config.get_installations().iter().enumerate() {
                println!("{}. {} - {}", i + 1, installation.display_name, installation.path.display());
            }
            println!("Enter number to play, 'a' to add installation, or 'q' to quit:");
            
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                let input = input.trim();
                
                if input == "q" {
                    return UIResult::Exit;
                } else if input == "a" {
                    self.add_installation_prompt();
                    return UIResult::None;
                } else if let Ok(num) = input.parse::<usize>() {
                    if num > 0 && num <= self.config.get_installations().len() {
                        if let Some((_, installation)) = self.config.get_installations().iter().nth(num - 1) {
                            return UIResult::PlayGame(installation.clone());
                        }
                    }
                }
            }
        }
        
        UIResult::None
    }

    fn show_auto_detected(&mut self) {
        if !self.found_installations.is_empty() {
            println!("Auto-detected installations:");
            for (i, path) in self.found_installations.iter().enumerate() {
                println!("{}. {}", i + 1, path.display());
            }
            println!("Enter number to add, 'b' to browse, or 'q' to quit:");
            
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                let input = input.trim();
                
                if input == "q" {
                } else if input == "b" {
                    self.browse_for_installation();
                } else if let Ok(num) = input.parse::<usize>() {
                    if num > 0 && num <= self.found_installations.len() {
                        if let Some(path) = self.found_installations.get(num - 1) {
                            self.add_installation(path.clone());
                        }
                    }
                }
            }
        } else {
            println!("Enter 'b' to browse for installation or 'q' to quit:");
            
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                let input = input.trim();
                
                if input == "b" {
                    self.browse_for_installation();
                }
            }
        }
    }

    fn add_installation_prompt(&mut self) {
        self.show_auto_detected();
    }

    fn browse_for_installation(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Call to Power Installation Directory")
            .pick_folder()
        {
            self.add_installation(path);
        }
    }

    fn add_installation(&mut self, path: PathBuf) {
        match InstallationValidator::create_installation(path) {
            Ok(installation) => {
                let id = format!("{}_{}", 
                    match installation.version {
                        GameVersion::CTP1 => "ctp1",
                        GameVersion::CTP2 => "ctp2",
                    },
                    installation.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                );
                
                println!("Added installation: {}", installation.display_name);
                self.config.add_installation(id, installation);
                
                if let Err(e) = self.config.save() {
                    println!("Failed to save config: {}", e);
                }
            }
            Err(e) => {
                println!("Invalid installation: {}", e);
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
}

#[derive(Debug)]
pub enum UIResult {
    None,
    PlayGame(GameInstallation),
    Exit,
}