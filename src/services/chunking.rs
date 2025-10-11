//! Text chunking utilities
//!
//! Provides functions to split text into chunks for embedding generation.

use crate::domain::ChunkStrategy;
use unicode_segmentation::UnicodeSegmentation;

/// Chunk text according to the specified strategy
pub fn chunk_text(text: &str, strategy: ChunkStrategy) -> Vec<String> {
    match strategy {
        ChunkStrategy::FixedSize { size, overlap } => chunk_fixed_size(text, size, overlap),
        ChunkStrategy::Semantic { max_size } => chunk_semantic(text, max_size),
    }
}

/// Chunk text using fixed size with overlap
fn chunk_fixed_size(text: &str, size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    if size <= overlap {
        // Invalid configuration - just return the whole text
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    let chars: Vec<&str> = text.graphemes(true).collect();

    while start < chars.len() {
        let end = (start + size).min(chars.len());
        let chunk: String = chars[start..end].iter().map(|s| *s).collect();

        if !chunk.trim().is_empty() {
            chunks.push(chunk);
        }

        // Move start forward by (size - overlap) for next chunk
        start += size.saturating_sub(overlap);

        // If we've reached the end, stop
        if end == chars.len() {
            break;
        }
    }

    chunks
}

/// Chunk text semantically by sentences and paragraphs
fn chunk_semantic(text: &str, max_size: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    // Split into paragraphs first
    for paragraph in text.split("\n\n") {
        let paragraph = paragraph.trim();
        if paragraph.is_empty() {
            continue;
        }

        // Split paragraph into sentences
        let sentences = split_into_sentences(paragraph);

        for sentence in sentences {
            let sentence_len = sentence.graphemes(true).count();

            // If adding this sentence would exceed max_size, start a new chunk
            let current_len = current_chunk.graphemes(true).count();

            if current_len > 0 && current_len + sentence_len > max_size {
                // Save current chunk and start new one
                if !current_chunk.trim().is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                }
                current_chunk = sentence.to_string();
            } else {
                // Add sentence to current chunk
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(sentence);
            }

            // If a single sentence is longer than max_size, split it
            if current_chunk.graphemes(true).count() > max_size {
                let split_chunks = chunk_fixed_size(&current_chunk, max_size, max_size / 10);
                for chunk in split_chunks {
                    if !chunk.trim().is_empty() {
                        chunks.push(chunk);
                    }
                }
                current_chunk.clear();
            }
        }

        // Add paragraph break context if we have a current chunk
        if !current_chunk.is_empty() && !current_chunk.ends_with('\n') {
            current_chunk.push('\n');
        }
    }

    // Add any remaining text
    if !current_chunk.trim().is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }

    chunks
}

/// Split text into sentences (simple implementation)
fn split_into_sentences(text: &str) -> Vec<&str> {
    let mut sentences = Vec::new();
    let mut start = 0;

    for (i, grapheme) in text.grapheme_indices(true) {
        if grapheme == "." || grapheme == "!" || grapheme == "?" {
            // Look ahead to see if this is really end of sentence
            let next_pos = i + grapheme.len();
            let is_end = if next_pos < text.len() {
                let next_char = &text[next_pos..next_pos + 1];
                next_char == " " || next_char == "\n" || next_char == "\t"
            } else {
                true
            };

            if is_end {
                let sentence = &text[start..next_pos];
                if !sentence.trim().is_empty() {
                    sentences.push(sentence);
                }
                start = next_pos;
            }
        }
    }

    // Add remaining text
    if start < text.len() {
        let remaining = &text[start..];
        if !remaining.trim().is_empty() {
            sentences.push(remaining);
        }
    }

    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_fixed_size_simple() {
        let text = "Hello world! This is a test.";
        let chunks = chunk_fixed_size(text, 10, 2);

        assert!(chunks.len() > 0);
        assert!(chunks[0].len() <= 10);
    }

    #[test]
    fn test_chunk_fixed_size_with_overlap() {
        let text = "0123456789"; // 10 chars
        let chunks = chunk_fixed_size(text, 5, 2);

        // With size=5 and overlap=2, step = 5-2 = 3
        // Should get: "01234" (0-4), "34567" (3-7), "6789" (6-9)
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "01234");
        assert_eq!(chunks[1], "34567");
        assert_eq!(chunks[2], "6789");
    }

    #[test]
    fn test_chunk_fixed_size_empty() {
        let chunks = chunk_fixed_size("", 10, 2);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunk_semantic() {
        let text = "This is sentence one. This is sentence two. This is sentence three.";
        let chunks = chunk_semantic(text, 30);

        assert!(chunks.len() > 0);
        for chunk in &chunks {
            assert!(chunk.graphemes(true).count() <= 35); // Small buffer for word boundaries
        }
    }

    #[test]
    fn test_chunk_semantic_paragraphs() {
        let text = "Paragraph one.\n\nParagraph two. Second sentence.\n\nParagraph three.";
        let chunks = chunk_semantic(text, 100);

        assert!(chunks.len() > 0);
    }

    #[test]
    fn test_chunk_text_wrapper() {
        let text = "Hello world! This is a test.";

        let fixed = chunk_text(
            text,
            ChunkStrategy::FixedSize {
                size: 10,
                overlap: 2,
            },
        );
        assert!(fixed.len() > 0);

        let semantic = chunk_text(text, ChunkStrategy::Semantic { max_size: 50 });
        assert!(semantic.len() > 0);
    }

    #[test]
    fn test_split_sentences() {
        let text = "First sentence. Second sentence! Third sentence? Fourth.";
        let sentences = split_into_sentences(text);

        assert!(sentences.len() >= 4);
    }
}
