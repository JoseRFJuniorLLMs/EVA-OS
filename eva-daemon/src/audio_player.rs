use crate::audio::AudioDevice;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Audio player for Gemini responses
pub struct AudioPlayer {
    device: AudioDevice,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new(device: AudioDevice) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { device })
    }

    /// Play audio response from base64 encoded data
    pub async fn play_response(&mut self, audio_data: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Decode base64
        let audio_bytes = BASE64.decode(audio_data)?;
        
        // Convert bytes to f32 samples
        let samples = self.bytes_to_samples(&audio_bytes);
        
        // Play through audio device
        self.device.play(&samples).await?;
        
        Ok(())
    }

    /// Play raw PCM audio bytes
    pub async fn play_pcm(&mut self, audio_bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let samples = self.bytes_to_samples(audio_bytes);
        self.device.play(&samples).await?;
        Ok(())
    }

    /// Convert bytes to f32 samples (16-bit PCM)
    ///
    /// # Safety
    /// Handles odd-length byte arrays safely by truncating to even length.
    /// This prevents potential panics from malformed audio data.
    fn bytes_to_samples(&self, bytes: &[u8]) -> Vec<f32> {
        // Ensure even number of bytes (16-bit samples = 2 bytes each)
        let valid_len = bytes.len() - (bytes.len() % 2);

        if valid_len != bytes.len() {
            eprintln!(
                "[AudioPlayer] Warning: Odd byte count ({}), truncating to {}",
                bytes.len(),
                valid_len
            );
        }

        if valid_len == 0 {
            return Vec::new();
        }

        bytes[..valid_len]
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / i16::MAX as f32
            })
            .collect()
    }

    /// Play text-to-speech (for testing without audio data)
    pub async fn speak_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔊 EVA: {}", text);
        
        // TODO: In future, convert text to speech locally
        // For now, just print the text
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_samples() {
        let device = AudioDevice::new().unwrap();
        let player = AudioPlayer::new(device).unwrap();

        // Test data: 4 bytes = 2 samples
        let bytes = vec![0x00, 0x00, 0xFF, 0x7F]; // [0, 32767]
        let samples = player.bytes_to_samples(&bytes);

        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0], 0.0);
        assert!((samples[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_bytes_to_samples_odd_length() {
        let device = AudioDevice::new().unwrap();
        let player = AudioPlayer::new(device).unwrap();

        // Test data: 5 bytes (odd) - should not panic, truncate to 4
        let bytes = vec![0x00, 0x00, 0xFF, 0x7F, 0xAB];
        let samples = player.bytes_to_samples(&bytes);

        assert_eq!(samples.len(), 2); // Only 2 samples from 4 valid bytes
    }

    #[test]
    fn test_bytes_to_samples_empty() {
        let device = AudioDevice::new().unwrap();
        let player = AudioPlayer::new(device).unwrap();

        // Empty input
        let samples = player.bytes_to_samples(&[]);
        assert!(samples.is_empty());

        // Single byte (odd, truncates to 0)
        let samples = player.bytes_to_samples(&[0x00]);
        assert!(samples.is_empty());
    }

    #[tokio::test]
    async fn test_speak_text() {
        let device = AudioDevice::new().unwrap();
        let mut player = AudioPlayer::new(device).unwrap();
        
        // Should not panic
        player.speak_text("Hello, world!").await.unwrap();
    }
}
