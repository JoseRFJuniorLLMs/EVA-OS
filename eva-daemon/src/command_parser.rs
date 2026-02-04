use std::collections::HashSet;

/// Command intent types
#[derive(Debug, Clone, PartialEq)]
pub enum CommandIntent {
    File(FileOperation),
    Process(ProcessOperation),
    System(SystemOperation),
    Network(NetworkOperation),
    Text(TextOperation),
    Unknown,
}

/// File operations
#[derive(Debug, Clone, PartialEq)]
pub enum FileOperation {
    Create { path: String, content: Option<String> },
    Delete { path: String },
    Copy { from: String, to: String },
    Move { from: String, to: String },
    List { path: Option<String> },
    Read { path: String },
}

/// Process operations
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessOperation {
    List,
    Start { name: String },
    Kill { pid: u32 },
}

/// System operations
#[derive(Debug, Clone, PartialEq)]
pub enum SystemOperation {
    MemoryInfo,
    DiskInfo,
    CpuInfo,
    Uptime,
}

/// Network operations
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkOperation {
    GetIP,
    Ping { host: String },
}

/// Text operations
#[derive(Debug, Clone, PartialEq)]
pub enum TextOperation {
    Type { text: String },
    Select,
    Copy,
    Paste,
}

/// Command parser
pub struct CommandParser {
    whitelist: HashSet<String>,
}

impl CommandParser {
    /// Create a new command parser
    pub fn new() -> Self {
        let mut whitelist = HashSet::new();
        
        // File operations
        whitelist.insert("create".to_string());
        whitelist.insert("delete".to_string());
        whitelist.insert("copy".to_string());
        whitelist.insert("move".to_string());
        whitelist.insert("list".to_string());
        whitelist.insert("read".to_string());
        
        // Process operations
        whitelist.insert("start".to_string());
        whitelist.insert("kill".to_string());
        whitelist.insert("processes".to_string());
        
        // System operations
        whitelist.insert("memory".to_string());
        whitelist.insert("disk".to_string());
        whitelist.insert("cpu".to_string());
        
        // Network operations
        whitelist.insert("ip".to_string());
        whitelist.insert("ping".to_string());
        
        // Text operations
        whitelist.insert("type".to_string());
        
        Self { whitelist }
    }

    /// Parse text into command intent
    pub fn parse(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        let text_lower = text.to_lowercase();
        
        // File operations
        if text_lower.contains("create") && text_lower.contains("file") {
            return self.parse_file_create(&text_lower);
        }
        
        if text_lower.contains("delete") && text_lower.contains("file") {
            return self.parse_file_delete(&text_lower);
        }
        
        if text_lower.contains("copy") {
            return self.parse_file_copy(&text_lower);
        }
        
        if text_lower.contains("move") {
            return self.parse_file_move(&text_lower);
        }
        
        if text_lower.contains("list") || text_lower.contains("show") && text_lower.contains("file") {
            return self.parse_file_list(&text_lower);
        }
        
        if text_lower.contains("read") && text_lower.contains("file") {
            return self.parse_file_read(&text_lower);
        }
        
        // Process operations
        if text_lower.contains("process") || text_lower.contains("running") {
            return Ok(CommandIntent::Process(ProcessOperation::List));
        }
        
        if text_lower.contains("start") || text_lower.contains("open") || text_lower.contains("launch") {
            return self.parse_process_start(&text_lower);
        }
        
        // System operations
        if text_lower.contains("memory") || text_lower.contains("ram") {
            return Ok(CommandIntent::System(SystemOperation::MemoryInfo));
        }
        
        if text_lower.contains("disk") || text_lower.contains("storage") {
            return Ok(CommandIntent::System(SystemOperation::DiskInfo));
        }
        
        if text_lower.contains("cpu") || text_lower.contains("processor") {
            return Ok(CommandIntent::System(SystemOperation::CpuInfo));
        }
        
        // Network operations
        if text_lower.contains("ip") && text_lower.contains("address") {
            return Ok(CommandIntent::Network(NetworkOperation::GetIP));
        }
        
        if text_lower.contains("ping") {
            return self.parse_network_ping(&text_lower);
        }
        
        // Text operations
        if text_lower.contains("type") {
            return self.parse_text_type(&text_lower);
        }
        
        Ok(CommandIntent::Unknown)
    }

    // File operation parsers
    fn parse_file_create(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        // Extract filename: "create a file called test.txt"
        let path = if let Some(idx) = text.find("called ") {
            text[idx + 7..].trim().split_whitespace().next().unwrap_or("untitled.txt").to_string()
        } else if let Some(idx) = text.find("named ") {
            text[idx + 6..].trim().split_whitespace().next().unwrap_or("untitled.txt").to_string()
        } else {
            "untitled.txt".to_string()
        };
        
        Ok(CommandIntent::File(FileOperation::Create { path, content: None }))
    }

    fn parse_file_delete(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        let path = if let Some(idx) = text.find("file ") {
            text[idx + 5..].trim().split_whitespace().next().unwrap_or("").to_string()
        } else {
            "".to_string()
        };
        
        if path.is_empty() {
            return Err("No filename specified".into());
        }
        
        Ok(CommandIntent::File(FileOperation::Delete { path }))
    }

    fn parse_file_copy(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        // "copy file1.txt to file2.txt"
        if let Some(to_idx) = text.find(" to ") {
            let from = text[..to_idx].split_whitespace().last().unwrap_or("").to_string();
            let to = text[to_idx + 4..].trim().split_whitespace().next().unwrap_or("").to_string();
            
            if !from.is_empty() && !to.is_empty() {
                return Ok(CommandIntent::File(FileOperation::Copy { from, to }));
            }
        }
        
        Err("Invalid copy command format".into())
    }

    fn parse_file_move(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        // "move file1.txt to file2.txt"
        if let Some(to_idx) = text.find(" to ") {
            let from = text[..to_idx].split_whitespace().last().unwrap_or("").to_string();
            let to = text[to_idx + 4..].trim().split_whitespace().next().unwrap_or("").to_string();
            
            if !from.is_empty() && !to.is_empty() {
                return Ok(CommandIntent::File(FileOperation::Move { from, to }));
            }
        }
        
        Err("Invalid move command format".into())
    }

    fn parse_file_list(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        // "list files" or "list files in documents"
        let path = if let Some(idx) = text.find(" in ") {
            Some(text[idx + 4..].trim().split_whitespace().next().unwrap_or(".").to_string())
        } else {
            None
        };
        
        Ok(CommandIntent::File(FileOperation::List { path }))
    }

    fn parse_file_read(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        let path = if let Some(idx) = text.find("file ") {
            text[idx + 5..].trim().split_whitespace().next().unwrap_or("").to_string()
        } else {
            "".to_string()
        };
        
        if path.is_empty() {
            return Err("No filename specified".into());
        }
        
        Ok(CommandIntent::File(FileOperation::Read { path }))
    }

    // Process operation parsers
    fn parse_process_start(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        // "start calculator" or "open notepad"
        let name = text.split_whitespace().last().unwrap_or("").to_string();
        
        if name.is_empty() {
            return Err("No program name specified".into());
        }
        
        Ok(CommandIntent::Process(ProcessOperation::Start { name }))
    }

    // Network operation parsers
    fn parse_network_ping(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        // "ping google.com"
        let host = text.split_whitespace().last().unwrap_or("localhost").to_string();
        
        Ok(CommandIntent::Network(NetworkOperation::Ping { host }))
    }

    // Text operation parsers
    fn parse_text_type(&self, text: &str) -> Result<CommandIntent, Box<dyn std::error::Error>> {
        // "type hello world" or "type 'hello world'"
        let text_to_type = if let Some(idx) = text.find("type ") {
            text[idx + 5..].trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string()
        } else {
            "".to_string()
        };
        
        if text_to_type.is_empty() {
            return Err("No text specified".into());
        }
        
        Ok(CommandIntent::Text(TextOperation::Type { text: text_to_type }))
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_create() {
        let parser = CommandParser::new();
        let result = parser.parse("create a file called test.txt").unwrap();
        
        assert!(matches!(result, CommandIntent::File(FileOperation::Create { .. })));
    }

    #[test]
    fn test_parse_file_list() {
        let parser = CommandParser::new();
        let result = parser.parse("list files").unwrap();
        
        assert!(matches!(result, CommandIntent::File(FileOperation::List { .. })));
    }

    #[test]
    fn test_parse_memory_info() {
        let parser = CommandParser::new();
        let result = parser.parse("show memory usage").unwrap();
        
        assert!(matches!(result, CommandIntent::System(SystemOperation::MemoryInfo)));
    }

    #[test]
    fn test_parse_process_list() {
        let parser = CommandParser::new();
        let result = parser.parse("show running processes").unwrap();
        
        assert!(matches!(result, CommandIntent::Process(ProcessOperation::List)));
    }

    #[test]
    fn test_parse_unknown() {
        let parser = CommandParser::new();
        let result = parser.parse("hello world").unwrap();
        
        assert!(matches!(result, CommandIntent::Unknown));
    }
}
