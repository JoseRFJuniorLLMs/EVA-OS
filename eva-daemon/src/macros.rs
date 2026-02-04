use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Voice macro definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMacro {
    pub name: String,
    pub steps: Vec<MacroStep>,
    pub created_at: SystemTime,
}

/// Macro step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroStep {
    pub command: String,
    pub delay_ms: u64,
}

/// Macro manager
pub struct MacroManager {
    macros: HashMap<String, VoiceMacro>,
    recording: Option<VoiceMacro>,
    config_path: PathBuf,
}

impl MacroManager {
    /// Create a new macro manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let macros = Self::load_macros(&config_path)?;

        Ok(Self {
            macros,
            recording: None,
            config_path,
        })
    }

    /// Start recording a macro
    pub fn start_recording(&mut self, name: String) {
        self.recording = Some(VoiceMacro {
            name,
            steps: Vec::new(),
            created_at: SystemTime::now(),
        });
    }

    /// Add a step to the current recording
    pub fn add_step(&mut self, command: String, delay_ms: u64) {
        if let Some(ref mut macro_rec) = self.recording {
            macro_rec.steps.push(MacroStep { command, delay_ms });
        }
    }

    /// Stop recording and return the macro
    pub fn stop_recording(&mut self) -> Result<VoiceMacro, Box<dyn std::error::Error>> {
        self.recording
            .take()
            .ok_or_else(|| "No recording in progress".into())
    }

    /// Save a macro
    pub fn save_macro(&mut self, macro_rec: VoiceMacro) -> Result<(), Box<dyn std::error::Error>> {
        self.macros.insert(macro_rec.name.clone(), macro_rec);
        self.save_to_disk()
    }

    /// Delete a macro
    pub fn delete_macro(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.macros.remove(name);
        self.save_to_disk()
    }

    /// Get a macro by name
    pub fn get_macro(&self, name: &str) -> Option<&VoiceMacro> {
        self.macros.get(name)
    }

    /// Play a macro (returns commands to execute)
    pub async fn play_macro(&self, name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let macro_rec = self
            .macros
            .get(name)
            .ok_or_else(|| format!("Macro '{}' not found", name))?;

        let mut results = Vec::new();

        for step in &macro_rec.steps {
            tokio::time::sleep(Duration::from_millis(step.delay_ms)).await;
            results.push(step.command.clone());
        }

        Ok(results)
    }

    /// Get macro count
    pub fn count(&self) -> usize {
        self.macros.len()
    }

    /// List all macros
    pub fn list_macros(&self) -> Vec<&VoiceMacro> {
        self.macros.values().collect()
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    /// Load macros from disk
    fn load_macros(
        path: &PathBuf,
    ) -> Result<HashMap<String, VoiceMacro>, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(path)?;
        let macros: Vec<VoiceMacro> = serde_json::from_str(&content)?;

        Ok(macros
            .into_iter()
            .map(|m| (m.name.clone(), m))
            .collect())
    }

    /// Save macros to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let macros: Vec<_> = self.macros.values().cloned().collect();
        let json = serde_json::to_string_pretty(&macros)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Get config file path
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            let home = std::env::var("USERPROFILE")?;
            Ok(PathBuf::from(home).join(".eva").join("macros.json"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join(".eva").join("macros.json"))
        }
    }
}

impl Default for MacroManager {
    fn default() -> Self {
        Self::new().expect("Failed to create macro manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_stop_recording() {
        let mut mgr = MacroManager::new().unwrap();

        mgr.start_recording("test_macro".to_string());
        assert!(mgr.is_recording());

        mgr.add_step("command 1".to_string(), 100);
        mgr.add_step("command 2".to_string(), 200);

        let macro_rec = mgr.stop_recording().unwrap();
        assert_eq!(macro_rec.name, "test_macro");
        assert_eq!(macro_rec.steps.len(), 2);
        assert!(!mgr.is_recording());
    }

    #[test]
    fn test_save_macro() {
        let mut mgr = MacroManager::new().unwrap();

        let macro_rec = VoiceMacro {
            name: "test".to_string(),
            steps: vec![MacroStep {
                command: "test command".to_string(),
                delay_ms: 100,
            }],
            created_at: SystemTime::now(),
        };

        mgr.save_macro(macro_rec).unwrap();
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_delete_macro() {
        let mut mgr = MacroManager::new().unwrap();

        let macro_rec = VoiceMacro {
            name: "delete_me".to_string(),
            steps: vec![],
            created_at: SystemTime::now(),
        };

        mgr.save_macro(macro_rec).unwrap();
        assert_eq!(mgr.count(), 1);

        mgr.delete_macro("delete_me").unwrap();
        assert_eq!(mgr.count(), 0);
    }

    #[tokio::test]
    async fn test_play_macro() {
        let mut mgr = MacroManager::new().unwrap();

        let macro_rec = VoiceMacro {
            name: "play_test".to_string(),
            steps: vec![
                MacroStep {
                    command: "cmd1".to_string(),
                    delay_ms: 10,
                },
                MacroStep {
                    command: "cmd2".to_string(),
                    delay_ms: 10,
                },
            ],
            created_at: SystemTime::now(),
        };

        mgr.save_macro(macro_rec).unwrap();

        let commands = mgr.play_macro("play_test").await.unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], "cmd1");
        assert_eq!(commands[1], "cmd2");
    }
}
