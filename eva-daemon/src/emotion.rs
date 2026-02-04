use std::collections::HashMap;
use std::fmt;

/// Emotion types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emotion {
    Happy,
    Sad,
    Angry,
    Neutral,
    Excited,
    Confused,
    Grateful,
    Frustrated,
}

impl fmt::Display for Emotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Emotion::Happy => write!(f, "Happy"),
            Emotion::Sad => write!(f, "Sad"),
            Emotion::Angry => write!(f, "Angry"),
            Emotion::Neutral => write!(f, "Neutral"),
            Emotion::Excited => write!(f, "Excited"),
            Emotion::Confused => write!(f, "Confused"),
            Emotion::Grateful => write!(f, "Grateful"),
            Emotion::Frustrated => write!(f, "Frustrated"),
        }
    }
}

/// Emotion detector
pub struct EmotionDetector {
    keywords: HashMap<Emotion, Vec<String>>,
}

impl EmotionDetector {
    /// Create a new emotion detector
    pub fn new() -> Self {
        let mut keywords = HashMap::new();

        keywords.insert(
            Emotion::Happy,
            vec![
                "happy".to_string(),
                "great".to_string(),
                "awesome".to_string(),
                "excellent".to_string(),
                "wonderful".to_string(),
                "good".to_string(),
                "nice".to_string(),
                "love".to_string(),
                "perfect".to_string(),
            ],
        );

        keywords.insert(
            Emotion::Sad,
            vec![
                "sad".to_string(),
                "unhappy".to_string(),
                "disappointed".to_string(),
                "bad".to_string(),
                "terrible".to_string(),
                "awful".to_string(),
                "depressed".to_string(),
            ],
        );

        keywords.insert(
            Emotion::Angry,
            vec![
                "angry".to_string(),
                "frustrated".to_string(),
                "annoyed".to_string(),
                "mad".to_string(),
                "furious".to_string(),
            ],
        );

        keywords.insert(
            Emotion::Excited,
            vec![
                "excited".to_string(),
                "amazing".to_string(),
                "wow".to_string(),
                "incredible".to_string(),
                "fantastic".to_string(),
            ],
        );

        keywords.insert(
            Emotion::Confused,
            vec![
                "confused".to_string(),
                "don't understand".to_string(),
                "what".to_string(),
                "unclear".to_string(),
                "puzzled".to_string(),
            ],
        );

        keywords.insert(
            Emotion::Grateful,
            vec![
                "thank".to_string(),
                "thanks".to_string(),
                "grateful".to_string(),
                "appreciate".to_string(),
            ],
        );

        keywords.insert(
            Emotion::Frustrated,
            vec![
                "frustrated".to_string(),
                "stuck".to_string(),
                "can't".to_string(),
                "won't work".to_string(),
            ],
        );

        Self { keywords }
    }

    /// Detect emotion from text
    pub fn detect(&self, text: &str) -> Emotion {
        let text_lower = text.to_lowercase();

        let mut scores = HashMap::new();

        for (emotion, words) in &self.keywords {
            let mut score = 0;
            for word in words {
                if text_lower.contains(word) {
                    score += 1;
                }
            }
            if score > 0 {
                scores.insert(emotion, score);
            }
        }

        // Return emotion with highest score
        scores
            .iter()
            .max_by_key(|(_, score)| *score)
            .map(|(emotion, _)| **emotion)
            .unwrap_or(Emotion::Neutral)
    }

    /// Get emotion confidence (0.0 to 1.0)
    pub fn detect_with_confidence(&self, text: &str) -> (Emotion, f32) {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        let total_words = words.len() as f32;

        if total_words == 0.0 {
            return (Emotion::Neutral, 1.0);
        }

        let mut scores = HashMap::new();

        for (emotion, keywords) in &self.keywords {
            let mut matches = 0;
            for keyword in keywords {
                if text_lower.contains(keyword) {
                    matches += 1;
                }
            }
            if matches > 0 {
                scores.insert(emotion, matches);
            }
        }

        if let Some((emotion, count)) = scores.iter().max_by_key(|(_, count)| *count) {
            let confidence = (*count as f32 / total_words).min(1.0);
            (**emotion, confidence)
        } else {
            (Emotion::Neutral, 1.0)
        }
    }
}

impl Default for EmotionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_happy() {
        let detector = EmotionDetector::new();
        assert_eq!(detector.detect("I'm so happy!"), Emotion::Happy);
        assert_eq!(detector.detect("This is great!"), Emotion::Happy);
    }

    #[test]
    fn test_detect_sad() {
        let detector = EmotionDetector::new();
        assert_eq!(detector.detect("This is terrible"), Emotion::Sad);
        assert_eq!(detector.detect("I'm so disappointed"), Emotion::Sad);
    }

    #[test]
    fn test_detect_angry() {
        let detector = EmotionDetector::new();
        assert_eq!(detector.detect("I'm so angry!"), Emotion::Angry);
        assert_eq!(detector.detect("This is frustrating"), Emotion::Angry);
    }

    #[test]
    fn test_detect_excited() {
        let detector = EmotionDetector::new();
        assert_eq!(detector.detect("Wow, this is amazing!"), Emotion::Excited);
    }

    #[test]
    fn test_detect_confused() {
        let detector = EmotionDetector::new();
        assert_eq!(detector.detect("I don't understand"), Emotion::Confused);
    }

    #[test]
    fn test_detect_grateful() {
        let detector = EmotionDetector::new();
        assert_eq!(detector.detect("Thank you so much!"), Emotion::Grateful);
    }

    #[test]
    fn test_detect_neutral() {
        let detector = EmotionDetector::new();
        assert_eq!(detector.detect("The sky is blue"), Emotion::Neutral);
    }

    #[test]
    fn test_detect_with_confidence() {
        let detector = EmotionDetector::new();
        let (emotion, confidence) = detector.detect_with_confidence("I'm very happy and excited!");
        
        assert!(emotion == Emotion::Happy || emotion == Emotion::Excited);
        assert!(confidence > 0.0 && confidence <= 1.0);
    }
}
