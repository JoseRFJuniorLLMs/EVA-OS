pub mod capture;
pub mod ocr;
pub mod embeddings;
pub mod index;
pub mod storage;
pub mod search;
pub mod npu_delegate;

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TimeMachine {
    #[allow(dead_code)]
    capture: capture::ScreenCapture,
    ocr: ocr::OCREngine,
    embeddings: embeddings::EmbeddingEngine,
    index: Arc<RwLock<index::SemanticIndex>>,
    storage: storage::Storage,
    #[allow(dead_code)]
    npu: npu_delegate::NPUDelegate,
}

impl TimeMachine {
    /// Create a new TimeMachine instance
    ///
    /// # Security
    /// Encryption key is obtained from (in order of priority):
    /// 1. EVA_TIMEMACHINE_KEY environment variable
    /// 2. Machine-specific derived key (fallback for development)
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("[TimeMachine] Initializing...");

        // 1. Initialize NPU
        let npu = npu_delegate::NPUDelegate::new()?;

        // 2. Load Models
        let ocr = ocr::OCREngine::new(&npu).await?;
        let embeddings = embeddings::EmbeddingEngine::new(&npu).await?;

        // 3. Setup Storage (Encrypted)
        let mut storage = storage::Storage::new("~/.eva/timemachine").await?;

        // Get encryption key securely
        let encryption_key = Self::get_encryption_key()?;
        storage.set_encryption_key(&encryption_key)?;

        // 4. Setup Index
        let index = Arc::new(RwLock::new(index::SemanticIndex::new()?));

        let capture = capture::ScreenCapture::new();

        Ok(Self {
            capture,
            ocr,
            embeddings,
            index,
            storage,
            npu,
        })
    }

    /// Get encryption key from environment or derive from machine-specific data
    fn get_encryption_key() -> Result<String, Box<dyn std::error::Error>> {
        // Priority 1: Environment variable (for production/user-set key)
        if let Ok(key) = std::env::var("EVA_TIMEMACHINE_KEY") {
            if key.len() >= 16 {
                println!("[TimeMachine] Using encryption key from environment");
                return Ok(key);
            } else {
                eprintln!("[TimeMachine] Warning: EVA_TIMEMACHINE_KEY too short (min 16 chars), using fallback");
            }
        }

        // Priority 2: Derive from machine-specific data (username + hostname)
        // This is NOT secure for production but prevents hardcoded keys
        let username = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "eva_user".to_string());

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "eva_host".to_string());

        let derived_key = format!("eva_tm_{}_{}_secret", username, hostname);

        println!("[TimeMachine] Warning: Using machine-derived key. Set EVA_TIMEMACHINE_KEY for production!");

        Ok(derived_key)
    }
    
    pub async fn start_recording(&self) {
        println!("[TimeMachine] Recording started");
        
        loop {
            // Snapshot every 10s
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            
            if let Err(e) = self.capture_and_process().await {
                // Ignore transient errors (e.g. privacy block)
                if e.to_string().contains("blocked") {
                    println!("[TimeMachine] Snapshot filtered (Privacy)");
                } else {
                    eprintln!("[TimeMachine] Error: {}", e);
                }
            }
        }
    }
    
    async fn capture_and_process(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Capture (Privacy filtered)
        let screenshot = self.capture.take_screenshot()?;
        
        // 2. OCR
        let text = self.ocr.extract_text(&screenshot)?;
        
        // 3. Embed
        let embedding = self.embeddings.encode(&text)?;
        
        // 4. Storage (Encrypted)
        let screenshot_id = self.storage.save_screenshot(screenshot).await?;
        self.storage.save_metadata(screenshot_id, &text).await?;
        
        // 5. Index
        let mut idx = self.index.write().await;
        idx.add(screenshot_id, embedding, &text)?;
        
        Ok(())
    }
    
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<(u64, f32, String)>, Box<dyn std::error::Error>> {
        let query_vec = self.embeddings.encode(query)?;
        
        let idx = self.index.read().await;
        // Search returns (id, score)
        let results = idx.search(&query_vec, limit)?;
        
        let mut final_results = Vec::new();
        for (id, score) in results {
            let metadata = self.storage.load_metadata(id).await?;
            final_results.push((id, score, metadata.text));
        }
        
        Ok(final_results)
    }
}
