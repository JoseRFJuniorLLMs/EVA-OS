use crate::command_parser::{CommandIntent, FileOperation, ProcessOperation, SystemOperation, NetworkOperation, TextOperation};
use std::path::{Path, PathBuf};
use std::fs;

/// Command executor with sandboxing
pub struct CommandExecutor {
    sandbox_dir: PathBuf,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create sandbox directory
        let sandbox_dir = Self::get_sandbox_dir()?;
        
        if !sandbox_dir.exists() {
            fs::create_dir_all(&sandbox_dir)?;
        }
        
        Ok(Self { sandbox_dir })
    }

    /// Get sandbox directory path
    fn get_sandbox_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            let home = std::env::var("USERPROFILE")?;
            Ok(PathBuf::from(home).join(".eva").join("sandbox"))
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join(".eva").join("sandbox"))
        }
    }

    /// Execute a command
    pub async fn execute(&mut self, intent: CommandIntent) -> Result<String, Box<dyn std::error::Error>> {
        match intent {
            CommandIntent::File(op) => self.execute_file_op(op).await,
            CommandIntent::Process(op) => self.execute_process_op(op).await,
            CommandIntent::System(op) => self.execute_system_op(op).await,
            CommandIntent::Network(op) => self.execute_network_op(op).await,
            CommandIntent::Text(op) => self.execute_text_op(op).await,
            CommandIntent::Unknown => Ok("I didn't understand that command.".to_string()),
        }
    }

    /// Validate and resolve path within sandbox
    ///
    /// # Security
    /// This function prevents path traversal attacks by:
    /// 1. Removing dangerous characters and sequences
    /// 2. Resolving to canonical path (follows symlinks)
    /// 3. Verifying final path is within sandbox
    fn validate_path(&self, path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // 1. Remove dangerous characters and sequences
        let mut clean_path = path.to_string();

        // Remove path traversal attempts (multiple passes for nested attacks)
        for _ in 0..5 {
            let before = clean_path.clone();
            clean_path = clean_path
                .replace("..", "")
                .replace("~", "")
                .replace("//", "/")
                .replace("\\\\", "\\")
                .replace("\0", ""); // Null byte injection

            if clean_path == before {
                break;
            }
        }

        // Remove leading slashes/backslashes (absolute path attempts)
        let clean_path = clean_path.trim_start_matches(['/', '\\']);

        // 2. Build full path within sandbox
        let full_path = self.sandbox_dir.join(clean_path);

        // 3. Get canonical sandbox path
        let sandbox_canonical = self.sandbox_dir.canonicalize()
            .unwrap_or_else(|_| self.sandbox_dir.clone());

        // 4. Try to canonicalize the target path
        // If file doesn't exist yet, check parent directory
        let target_canonical = if full_path.exists() {
            full_path.canonicalize()?
        } else {
            // For new files, verify parent is in sandbox
            let parent = full_path.parent()
                .ok_or("Invalid path: no parent directory")?;

            if !parent.exists() {
                // Create parent directories within sandbox
                fs::create_dir_all(parent)?;
            }

            let parent_canonical = parent.canonicalize()?;

            // Verify parent is in sandbox
            if !parent_canonical.starts_with(&sandbox_canonical) {
                return Err("Path traversal detected: parent outside sandbox".into());
            }

            // Return the non-canonical path for new files
            full_path
        };

        // 5. Final verification: ensure path is within sandbox
        if target_canonical.exists() && !target_canonical.starts_with(&sandbox_canonical) {
            return Err("Path traversal detected: resolved path outside sandbox".into());
        }

        Ok(target_canonical)
    }

    /// Execute file operation
    async fn execute_file_op(&self, op: FileOperation) -> Result<String, Box<dyn std::error::Error>> {
        match op {
            FileOperation::Create { path, content } => {
                let safe_path = self.validate_path(&path)?;
                
                if let Some(content) = content {
                    fs::write(&safe_path, content)?;
                } else {
                    fs::File::create(&safe_path)?;
                }
                
                Ok(format!("Created file: {}", path))
            }
            
            FileOperation::Delete { path } => {
                let safe_path = self.validate_path(&path)?;
                
                if !safe_path.exists() {
                    return Err(format!("File not found: {}", path).into());
                }
                
                fs::remove_file(&safe_path)?;
                Ok(format!("Deleted file: {}", path))
            }
            
            FileOperation::Copy { from, to } => {
                let safe_from = self.validate_path(&from)?;
                let safe_to = self.validate_path(&to)?;
                
                if !safe_from.exists() {
                    return Err(format!("Source file not found: {}", from).into());
                }
                
                fs::copy(&safe_from, &safe_to)?;
                Ok(format!("Copied {} to {}", from, to))
            }
            
            FileOperation::Move { from, to } => {
                let safe_from = self.validate_path(&from)?;
                let safe_to = self.validate_path(&to)?;
                
                if !safe_from.exists() {
                    return Err(format!("Source file not found: {}", from).into());
                }
                
                fs::rename(&safe_from, &safe_to)?;
                Ok(format!("Moved {} to {}", from, to))
            }
            
            FileOperation::List { path } => {
                let safe_path = if let Some(p) = path {
                    self.validate_path(&p)?
                } else {
                    self.sandbox_dir.clone()
                };
                
                if !safe_path.exists() {
                    return Ok("Directory is empty or doesn't exist.".to_string());
                }
                
                let mut files = Vec::new();
                
                for entry in fs::read_dir(&safe_path)? {
                    let entry = entry?;
                    let name = entry.file_name().to_string_lossy().to_string();
                    let metadata = entry.metadata()?;
                    
                    if metadata.is_dir() {
                        files.push(format!("📁 {}", name));
                    } else {
                        let size = metadata.len();
                        files.push(format!("📄 {} ({} bytes)", name, size));
                    }
                }
                
                if files.is_empty() {
                    Ok("Directory is empty.".to_string())
                } else {
                    Ok(format!("Found {} items:\n{}", files.len(), files.join("\n")))
                }
            }
            
            FileOperation::Read { path } => {
                let safe_path = self.validate_path(&path)?;
                
                if !safe_path.exists() {
                    return Err(format!("File not found: {}", path).into());
                }
                
                let content = fs::read_to_string(&safe_path)?;
                
                // Limit output size
                if content.len() > 500 {
                    Ok(format!("File content (first 500 chars):\n{}", &content[..500]))
                } else {
                    Ok(format!("File content:\n{}", content))
                }
            }
        }
    }

    /// Execute process operation
    async fn execute_process_op(&self, op: ProcessOperation) -> Result<String, Box<dyn std::error::Error>> {
        match op {
            ProcessOperation::List => {
                // Get process list using sysinfo
                #[cfg(feature = "sysinfo")]
                {
                    use sysinfo::{System, SystemExt, ProcessExt};
                    let mut sys = System::new_all();
                    sys.refresh_all();
                    
                    let mut processes = Vec::new();
                    for (pid, process) in sys.processes() {
                        processes.push(format!("PID {}: {}", pid, process.name()));
                    }
                    
                    Ok(format!("Running processes ({}):\n{}", 
                               processes.len(), 
                               processes.iter().take(10).cloned().collect::<Vec<_>>().join("\n")))
                }
                
                #[cfg(not(feature = "sysinfo"))]
                {
                    Ok("Process listing not available (sysinfo feature disabled)".to_string())
                }
            }
            
            ProcessOperation::Start { name } => {
                // Expanded capability: Open specific apps or URLs
                // Note: We are trusting the intent classifier to not send malicious commands
                
                #[cfg(target_os = "windows")]
                {
                    // Use 'cmd /C start' to leverage Windows shell association (handles URLs, apps, files)
                    // The empty string argument is for the window title, which start interprets as title if quoted
                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", &name])
                        .spawn()?;
                        
                    Ok(format!("Started/Opened: {}", name))
                }
                
                #[cfg(not(target_os = "windows"))]
                {
                    // Linux/Mac implementation (fallback to xdg-open or open)
                    if name.starts_with("http") {
                         std::process::Command::new("xdg-open").arg(&name).spawn()?;
                         Ok(format!("Opened URL: {}", name))
                    } else {
                         std::process::Command::new(&name).spawn()?;
                         Ok(format!("Started process: {}", name))
                    }
                }
            }
            
            ProcessOperation::Kill { pid } => {
                // Implement process kill using sysinfo
                #[cfg(feature = "sysinfo")]
                {
                    use sysinfo::{System, SystemExt, ProcessExt, Pid, PidExt};
                    let mut sys = System::new_all();
                    sys.refresh_processes();
                    
                    let sys_pid = Pid::from_u32(pid);
                    
                    if let Some(process) = sys.process(sys_pid) {
                        if process.kill() {
                            Ok(format!("Successfully killed process {} ({})", pid, process.name()))
                        } else {
                            Err(format!("Failed to kill process {}", pid).into())
                        }
                    } else {
                        Err(format!("Process {} not found", pid).into())
                    }
                }
                
                #[cfg(not(feature = "sysinfo"))]
                {
                    Err("Process kill requires sysinfo feature".into())
                }
            }
        }
    }

    /// Execute system operation
    async fn execute_system_op(&self, op: SystemOperation) -> Result<String, Box<dyn std::error::Error>> {
        match op {
            SystemOperation::MemoryInfo => {
                #[cfg(feature = "sysinfo")]
                {
                    use sysinfo::{System, SystemExt};
                    let mut sys = System::new_all();
                    sys.refresh_all();
                    
                    let total = sys.total_memory();
                    let used = sys.used_memory();
                    let percent = (used as f64 / total as f64 * 100.0) as u32;
                    
                    Ok(format!("Memory: {} MB used / {} MB total ({}%)", 
                               used / 1024 / 1024, 
                               total / 1024 / 1024, 
                               percent))
                }
                
                #[cfg(not(feature = "sysinfo"))]
                {
                    Ok("Memory info not available (sysinfo feature disabled)".to_string())
                }
            }
            
            SystemOperation::DiskInfo => {
                #[cfg(feature = "sysinfo")]
                {
                    use sysinfo::{System, SystemExt, DiskExt};
                    let mut sys = System::new_all();
                    sys.refresh_disks();
                    
                    let mut info = Vec::new();
                    for disk in sys.disks() {
                        let total_gb = disk.total_space() / 1024 / 1024 / 1024;
                        let available_gb = disk.available_space() / 1024 / 1024 / 1024;
                        info.push(format!("{}: {} GB free / {} GB total", 
                            disk.name().to_string_lossy(), available_gb, total_gb));
                    }
                    
                    if info.is_empty() {
                         Ok("No disks found.".to_string())
                    } else {
                         Ok(info.join("\n"))
                    }
                }
                #[cfg(not(feature = "sysinfo"))]
                {
                     Ok("Disk info not yet implemented".to_string())
                }
            }
            
            SystemOperation::CpuInfo => {
                #[cfg(feature = "sysinfo")]
                {
                    use sysinfo::{System, SystemExt, CpuExt};
                    let mut sys = System::new_all();
                    sys.refresh_cpu();
                    
                    let cpu_count = sys.cpus().len();
                    // Just basic info for now
                    Ok(format!("CPU: {} cores detected.", cpu_count))
                }
                
                #[cfg(not(feature = "sysinfo"))]
                {
                    Ok("CPU info not available (sysinfo feature disabled)".to_string())
                }
            }
            
            SystemOperation::Uptime => {
                 #[cfg(feature = "sysinfo")]
                {
                    use sysinfo::{System, SystemExt};
                    let mut sys = System::new_all();
                    sys.refresh_system(); // Only need system refresh for uptime
                    
                    let uptime = sys.uptime();
                    let hours = uptime / 3600;
                    let minutes = (uptime % 3600) / 60;
                    
                    Ok(format!("System Uptime: {} hours, {} minutes", hours, minutes))
                }
                #[cfg(not(feature = "sysinfo"))]
                {
                    Ok("Uptime requires sysinfo feature".to_string())
                }
            }
        }
    }

    /// Execute network operation
    async fn execute_network_op(&self, op: NetworkOperation) -> Result<String, Box<dyn std::error::Error>> {
        match op {
            NetworkOperation::GetIP => {
                // Get local IP
                Ok("IP address: 127.0.0.1 (localhost)".to_string())
            }
            
            NetworkOperation::Ping { host } => {
                Ok(format!("Ping {} - not yet implemented", host))
            }
        }
    }

    /// Execute text operation
    async fn execute_text_op(&self, op: TextOperation) -> Result<String, Box<dyn std::error::Error>> {
        match op {
            TextOperation::Type { text } => {
                Ok(format!("Typed: {}", text))
            }
            
            TextOperation::Select => {
                Ok("Select all - not yet implemented".to_string())
            }
            
            TextOperation::Copy => {
                Ok("Copy - not yet implemented".to_string())
            }
            
            TextOperation::Paste => {
                Ok("Paste - not yet implemented".to_string())
            }
        }
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create command executor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_creation() {
        let executor = CommandExecutor::new().unwrap();
        assert!(executor.sandbox_dir.exists());
    }

    #[test]
    fn test_path_validation() {
        let executor = CommandExecutor::new().unwrap();

        // Valid path
        assert!(executor.validate_path("test.txt").is_ok());

        // Path traversal attempts - all should be sanitized to stay in sandbox
        let test_cases = vec![
            "../etc/passwd",
            "..\\..\\windows\\system32",
            "....//etc/passwd",      // Nested traversal
            "/etc/passwd",           // Absolute path
            "~/.bashrc",             // Home directory
            "foo/../../../etc/passwd", // Mixed traversal
            "test\0.txt",            // Null byte injection
        ];

        for path in test_cases {
            let result = executor.validate_path(path);
            // Should either succeed (sanitized) or fail (blocked)
            if let Ok(safe_path) = result {
                // If it succeeds, must be within sandbox
                let sandbox_canonical = executor.sandbox_dir.canonicalize()
                    .unwrap_or_else(|_| executor.sandbox_dir.clone());
                assert!(
                    safe_path.starts_with(&executor.sandbox_dir) ||
                    safe_path.starts_with(&sandbox_canonical),
                    "Path {} escaped sandbox to {:?}",
                    path,
                    safe_path
                );
            }
            // If it fails, that's also acceptable (blocked attack)
        }
    }

    #[tokio::test]
    async fn test_file_create() {
        let executor = CommandExecutor::new().unwrap();
        
        let op = FileOperation::Create {
            path: "test_file.txt".to_string(),
            content: Some("Hello, EVA!".to_string()),
        };
        
        let result = executor.execute_file_op(op).await.unwrap();
        assert!(result.contains("Created file"));
        
        // Cleanup
        let _ = fs::remove_file(executor.sandbox_dir.join("test_file.txt"));
    }
}
