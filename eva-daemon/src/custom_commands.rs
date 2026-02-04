use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Custom command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub trigger: String,
    pub action: CommandAction,
    pub description: String,
}

/// Command action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandAction {
    ExecuteShell(String),
    RunMacro(String),
    SendText(String),
    Custom(String),
}

/// Custom command manager
pub struct CustomCommandManager {
    commands: HashMap<String, CustomCommand>,
    config_path: PathBuf,
}

impl CustomCommandManager {
    /// Create a new custom command manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let commands = Self::load_commands(&config_path)?;

        Ok(Self {
            commands,
            config_path,
        })
    }

    /// Add a custom command
    pub fn add_command(&mut self, cmd: CustomCommand) -> Result<(), Box<dyn std::error::Error>> {
        self.commands.insert(cmd.trigger.clone(), cmd);
        self.save_commands()
    }

    /// Remove a custom command
    pub fn remove_command(&mut self, trigger: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.commands.remove(trigger);
        self.save_commands()
    }

    /// Find a command by text
    pub fn find_command(&self, text: &str) -> Option<&CustomCommand> {
        let text_lower = text.to_lowercase();

        // Check for exact match
        if let Some(cmd) = self.commands.get(&text_lower) {
            return Some(cmd);
        }

        // Check for partial match
        for (trigger, cmd) in &self.commands {
            if text_lower.contains(trigger) {
                return Some(cmd);
            }
        }

        None
    }

    /// Get command count
    pub fn count(&self) -> usize {
        self.commands.len()
    }

    /// List all commands
    pub fn list_commands(&self) -> Vec<&CustomCommand> {
        self.commands.values().collect()
    }

    /// Load commands from disk
    fn load_commands(
        path: &PathBuf,
    ) -> Result<HashMap<String, CustomCommand>, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(path)?;
        let commands: Vec<CustomCommand> = serde_json::from_str(&content)?;

        Ok(commands
            .into_iter()
            .map(|cmd| (cmd.trigger.clone(), cmd))
            .collect())
    }

    /// Save commands to disk
    fn save_commands(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let commands: Vec<_> = self.commands.values().cloned().collect();
        let json = serde_json::to_string_pretty(&commands)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Get config file path
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            let home = std::env::var("USERPROFILE")?;
            Ok(PathBuf::from(home)
                .join(".eva")
                .join("custom_commands.json"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home)
                .join(".eva")
                .join("custom_commands.json"))
        }
    }
}

impl Default for CustomCommandManager {
    fn default() -> Self {
        Self::new().expect("Failed to create custom command manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_command() {
        let mut mgr = CustomCommandManager::new().unwrap();

        let cmd = CustomCommand {
            trigger: "good morning".to_string(),
            action: CommandAction::SendText("Hello!".to_string()),
            description: "Morning greeting".to_string(),
        };

        mgr.add_command(cmd).unwrap();
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_find_command() {
        let mut mgr = CustomCommandManager::new().unwrap();

        let cmd = CustomCommand {
            trigger: "test".to_string(),
            action: CommandAction::SendText("Test response".to_string()),
            description: "Test command".to_string(),
        };

        mgr.add_command(cmd).unwrap();

        // Exact match
        assert!(mgr.find_command("test").is_some());

        // Partial match
        assert!(mgr.find_command("this is a test").is_some());

        // No match
        assert!(mgr.find_command("nothing").is_none());
    }

    #[test]
    fn test_remove_command() {
        let mut mgr = CustomCommandManager::new().unwrap();

        let cmd = CustomCommand {
            trigger: "remove_me".to_string(),
            action: CommandAction::SendText("Test".to_string()),
            description: "Test".to_string(),
        };

        mgr.add_command(cmd).unwrap();
        assert_eq!(mgr.count(), 1);

        mgr.remove_command("remove_me").unwrap();
        assert_eq!(mgr.count(), 0);
    }
}
