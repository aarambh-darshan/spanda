//! Text splitting for staggered character/word animation.
//!
//! `SplitText` splits a string into characters and words with index metadata,
//! and can generate pre-staggered [`Timeline`]s for each. The DOM injection
//! methods (`inject_chars`, `inject_words`, `detect_lines`) require the
//! `wasm-dom` feature.
//!
//! # Example
//!
//! ```rust
//! use spanda::integrations::split_text::SplitText;
//!
//! let split = SplitText::from_str("Hello world");
//! assert_eq!(split.char_count(), 11);
//! assert_eq!(split.word_count(), 2);
//! ```

use crate::easing::Easing;
use crate::timeline::{stagger, Timeline};
use crate::tween::Tween;
use crate::traits::Animatable;

/// A single character with index metadata.
#[derive(Debug, Clone)]
pub struct SplitChar {
    /// The character.
    pub ch: char,
    /// Index within the full text (character position).
    pub index: usize,
    /// Which word this character belongs to.
    pub word_index: usize,
}

/// A single word.
#[derive(Debug, Clone)]
pub struct SplitWord {
    /// The word text.
    pub text: String,
    /// Word index (0-based).
    pub index: usize,
    /// First character index in the full text.
    pub char_start: usize,
    /// Exclusive end character index.
    pub char_end: usize,
}

/// Split text into characters and words for staggered animation.
#[derive(Debug, Clone)]
pub struct SplitText {
    chars: Vec<SplitChar>,
    words: Vec<SplitWord>,
    original: String,
}

impl SplitText {
    /// Split a string into characters and words. Works everywhere (no DOM).
    pub fn from_str(text: &str) -> Self {
        let mut chars = Vec::new();
        let mut words = Vec::new();
        let mut word_index = 0;
        let mut char_index = 0;

        for word_str in text.split_whitespace() {
            let char_start = text.find(word_str).unwrap_or(char_index);
            // Add space characters between words (if any)
            while char_index < char_start {
                if let Some(ch) = text.chars().nth(char_index) {
                    chars.push(SplitChar {
                        ch,
                        index: char_index,
                        word_index: if word_index > 0 { word_index - 1 } else { 0 },
                    });
                }
                char_index += 1;
            }

            let word_char_start = char_index;
            for ch in word_str.chars() {
                chars.push(SplitChar {
                    ch,
                    index: char_index,
                    word_index,
                });
                char_index += 1;
            }

            words.push(SplitWord {
                text: word_str.to_string(),
                index: word_index,
                char_start: word_char_start,
                char_end: char_index,
            });
            word_index += 1;
        }

        // Remaining characters (trailing spaces)
        while char_index < text.len() {
            if let Some(ch) = text.chars().nth(char_index) {
                chars.push(SplitChar {
                    ch,
                    index: char_index,
                    word_index: if word_index > 0 { word_index - 1 } else { 0 },
                });
            }
            char_index += 1;
        }

        Self {
            chars,
            words,
            original: text.to_string(),
        }
    }

    /// All split characters.
    pub fn chars(&self) -> &[SplitChar] {
        &self.chars
    }

    /// All split words.
    pub fn words(&self) -> &[SplitWord] {
        &self.words
    }

    /// Total character count (including spaces).
    pub fn char_count(&self) -> usize {
        self.chars.len()
    }

    /// Word count.
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// The original text.
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Create a staggered [`Timeline`] for each character.
    ///
    /// Each character gets a `Tween<T>` from `from` to `to` with the given
    /// duration and easing, staggered by `delay` seconds.
    pub fn stagger_chars<T: Animatable + Clone>(
        &self,
        from: T,
        to: T,
        duration: f32,
        delay: f32,
        easing: Easing,
    ) -> Timeline {
        let tweens: Vec<_> = (0..self.chars.len())
            .map(|_| {
                let t = Tween::new(from.clone(), to.clone())
                    .duration(duration)
                    .easing(easing.clone())
                    .build();
                (t, duration)
            })
            .collect();
        stagger(tweens, delay)
    }

    /// Create a staggered [`Timeline`] for each word.
    pub fn stagger_words<T: Animatable + Clone>(
        &self,
        from: T,
        to: T,
        duration: f32,
        delay: f32,
        easing: Easing,
    ) -> Timeline {
        let tweens: Vec<_> = (0..self.words.len())
            .map(|_| {
                let t = Tween::new(from.clone(), to.clone())
                    .duration(duration)
                    .easing(easing.clone())
                    .build();
                (t, duration)
            })
            .collect();
        stagger(tweens, delay)
    }

    /// Wrap each character in a `<span>` inside the parent element.
    ///
    /// Clears the parent's content first, then creates one `<span>` per
    /// character with `display: inline-block` for individual animation.
    /// Space characters become `&nbsp;`.
    #[cfg(feature = "wasm-dom")]
    pub fn inject_chars(&self, parent: &web_sys::Element) {
        parent.set_inner_html("");
        let doc = web_sys::window().unwrap().document().unwrap();

        for sc in &self.chars {
            let span = doc.create_element("span").unwrap();
            let _ = span.set_attribute("style", "display:inline-block");
            let _ = span.set_attribute("data-char-index", &sc.index.to_string());
            if sc.ch == ' ' {
                span.set_inner_html("&nbsp;");
            } else {
                span.set_text_content(Some(&sc.ch.to_string()));
            }
            let _ = parent.append_child(&span);
        }
    }

    /// Wrap each word in a `<span>` inside the parent element.
    ///
    /// Words are separated by spaces. Each `<span>` has `display: inline-block`.
    #[cfg(feature = "wasm-dom")]
    pub fn inject_words(&self, parent: &web_sys::Element) {
        parent.set_inner_html("");
        let doc = web_sys::window().unwrap().document().unwrap();

        for (i, sw) in self.words.iter().enumerate() {
            if i > 0 {
                // Add a space text node between words
                let space = doc.create_text_node(" ");
                let _ = parent.append_child(&space);
            }
            let span = doc.create_element("span").unwrap();
            let _ = span.set_attribute("style", "display:inline-block");
            let _ = span.set_attribute("data-word-index", &sw.index.to_string());
            span.set_text_content(Some(&sw.text));
            let _ = parent.append_child(&span);
        }
    }

    /// Detect visual lines by comparing `getBoundingClientRect().top` of
    /// injected char spans.
    ///
    /// **Call `inject_chars` first**, then call this on the same container.
    /// Returns groups of character indices, one group per visual line.
    ///
    /// This is expensive — call once, not every frame.
    #[cfg(feature = "wasm-dom")]
    pub fn detect_lines(container: &web_sys::Element) -> Vec<Vec<usize>> {
        let spans = container.children();
        let mut lines: Vec<Vec<usize>> = Vec::new();
        let mut current_top: Option<f32> = None;

        for i in 0..spans.length() {
            if let Some(span) = spans.item(i) {
                let rect = span.get_bounding_client_rect();
                let top = rect.top() as f32;

                match current_top {
                    Some(ct) if (top - ct).abs() < 2.0 => {
                        // Same line
                        if let Some(line) = lines.last_mut() {
                            line.push(i as usize);
                        }
                    }
                    _ => {
                        // New line
                        current_top = Some(top);
                        lines.push(vec![i as usize]);
                    }
                }
            }
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Update;

    #[test]
    fn split_basic() {
        let split = SplitText::from_str("Hello world");
        assert_eq!(split.word_count(), 2);
        assert_eq!(split.words()[0].text, "Hello");
        assert_eq!(split.words()[1].text, "world");
    }

    #[test]
    fn split_chars_count() {
        let split = SplitText::from_str("Hi");
        assert_eq!(split.char_count(), 2);
        assert_eq!(split.chars()[0].ch, 'H');
        assert_eq!(split.chars()[1].ch, 'i');
    }

    #[test]
    fn split_empty_string() {
        let split = SplitText::from_str("");
        assert_eq!(split.char_count(), 0);
        assert_eq!(split.word_count(), 0);
    }

    #[test]
    fn split_single_word() {
        let split = SplitText::from_str("Rust");
        assert_eq!(split.word_count(), 1);
        assert_eq!(split.chars()[0].word_index, 0);
    }

    #[test]
    fn stagger_chars_creates_timeline() {
        let split = SplitText::from_str("ABC");
        let mut timeline = split.stagger_chars(0.0_f32, 1.0, 0.5, 0.1, Easing::Linear);
        timeline.play();
        // Should not panic, and should be playable
        timeline.update(0.1);
    }

    #[test]
    fn stagger_words_creates_timeline() {
        let split = SplitText::from_str("One Two Three");
        let mut timeline = split.stagger_words(0.0_f32, 1.0, 0.5, 0.2, Easing::Linear);
        timeline.play();
        timeline.update(0.3);
    }
}
