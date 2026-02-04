use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// User profile with preferences and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub language: String,
    pub voice_speed: f32,
    pub wake_word_sensitivity: f32,
    pub custom_wake_word: Option<String>,
    pub preferences: HashMap<String, String>,
}

impl UserProfile {
    /// Create default user profile
    pub fn default() -> Self {
        Self {
            name: "User".to_string(),
            language: "en-US".to_string(),
            voice_speed: 1.0,
            wake_word_sensitivity: 0.6,
            custom_wake_word: None,
            preferences: HashMap::new(),
        }
    }

    /// Load user profile from disk
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_profile_path()?;

        if !path.exists() {
            let profile = Self::default();
            profile.save()?;
            return Ok(profile);
        }

        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Save user profile to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_profile_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Get profile file path
    fn get_profile_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            let home = std::env::var("USERPROFILE")?;
            Ok(PathBuf::from(home).join(".eva").join("profile.json"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join(".eva").join("profile.json"))
        }
    }

    /// Set a preference
    pub fn set_preference(&mut self, key: &str, value: &str) {
        self.preferences.insert(key.to_string(), value.to_string());
    }

    /// Get a preference
    pub fn get_preference(&self, key: &str) -> Option<&String> {
        self.preferences.get(key)
    }

    /// Update language
    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    /// Update wake word sensitivity
    pub fn set_wake_word_sensitivity(&mut self, sensitivity: f32) {
        self.wake_word_sensitivity = sensitivity.clamp(0.0, 1.0);
    }

    /// Update voice speed
    pub fn set_voice_speed(&mut self, speed: f32) {
        self.voice_speed = speed.clamp(0.5, 2.0);
    }

    /// Set custom wake word
    pub fn set_custom_wake_word(&mut self, wake_word: Option<String>) {
        self.custom_wake_word = wake_word;
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let profile = UserProfile::default();
        assert_eq!(profile.name, "User");
        assert_eq!(profile.language, "en-US");
        assert_eq!(profile.voice_speed, 1.0);
        assert_eq!(profile.wake_word_sensitivity, 0.6);
    }

    #[test]
    fn test_preferences() {
        let mut profile = UserProfile::default();
        
        profile.set_preference("theme", "dark");
        assert_eq!(profile.get_preference("theme"), Some(&"dark".to_string()));
        
        profile.set_preference("notifications", "enabled");
        assert_eq!(profile.get_preference("notifications"), Some(&"enabled".to_string()));
    }

    #[test]
    fn test_language_update() {
        let mut profile = UserProfile::default();
        profile.set_language("pt-BR");
        assert_eq!(profile.language, "pt-BR");
    }

    #[test]
    fn test_sensitivity_clamping() {
        let mut profile = UserProfile::default();
        
        profile.set_wake_word_sensitivity(1.5);
        assert_eq!(profile.wake_word_sensitivity, 1.0);
        
        profile.set_wake_word_sensitivity(-0.5);
        assert_eq!(profile.wake_word_sensitivity, 0.0);
    }

    #[test]
    fn test_voice_speed_clamping() {
        let mut profile = UserProfile::default();
        
        profile.set_voice_speed(3.0);
        assert_eq!(profile.voice_speed, 2.0);
        
        profile.set_voice_speed(0.1);
        assert_eq!(profile.voice_speed, 0.5);
    }
}
