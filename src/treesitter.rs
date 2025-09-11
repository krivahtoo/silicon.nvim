use crate::error::Error;
use std::collections::HashMap;
use syntect::highlighting::{Color, FontStyle, Style};

pub struct TreesitterHighlighter;

impl TreesitterHighlighter {
    pub fn new() -> Self {
        Self
    }

    /// Get tree-sitter highlights for the given code and filetype
    pub fn highlight_code(
        &self,
        code: &str,
        filetype: &str,
    ) -> Result<Vec<Vec<(Style, String)>>, Error> {
        // For now, implement a simplified version that uses basic highlighting
        // In a full implementation, this would integrate with Neovim's tree-sitter
        self.basic_highlight_fallback(code, filetype)
    }

    /// Basic highlighting fallback when tree-sitter is not available or fails
    fn basic_highlight_fallback(
        &self,
        code: &str,
        _filetype: &str,
    ) -> Result<Vec<Vec<(Style, String)>>, Error> {
        let lines: Vec<&str> = code.lines().collect();
        let mut result = Vec::new();

        for line in lines {
            let mut line_result = Vec::new();
            
            // Simple keyword-based highlighting
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut current_pos = 0;
            
            for word in words {
                // Find the position of this word in the line
                if let Some(word_pos) = line[current_pos..].find(word) {
                    let actual_pos = current_pos + word_pos;
                    
                    // Add any whitespace before the word
                    if actual_pos > current_pos {
                        let whitespace = &line[current_pos..actual_pos];
                        line_result.push((self.default_style(), whitespace.to_string()));
                    }
                    
                    // Add the word with appropriate highlighting
                    let style = self.get_word_style(word);
                    line_result.push((style, word.to_string()));
                    
                    current_pos = actual_pos + word.len();
                }
            }
            
            // Add any remaining text
            if current_pos < line.len() {
                let remaining = &line[current_pos..];
                line_result.push((self.default_style(), remaining.to_string()));
            }
            
            // If no words were found, add the entire line
            if line_result.is_empty() {
                line_result.push((self.default_style(), line.to_string()));
            }
            
            result.push(line_result);
        }

        Ok(result)
    }

    /// Get style for a specific word based on simple keyword matching
    fn get_word_style(&self, word: &str) -> Style {
        match word {
            // Keywords
            "def" | "class" | "import" | "from" | "if" | "else" | "elif" | "for" | "while" | 
            "try" | "except" | "finally" | "with" | "as" | "return" | "yield" | "break" | 
            "continue" | "pass" | "raise" | "assert" | "global" | "nonlocal" | "lambda" |
            "and" | "or" | "not" | "in" | "is" | "True" | "False" | "None" => {
                Style {
                    foreground: Color { r: 0x79, g: 0x79, b: 0xFF, a: 255 }, // Blue
                    background: Color { r: 0x27, g: 0x28, b: 0x22, a: 255 },
                    font_style: FontStyle::BOLD,
                }
            }
            // Check for string literals
            w if (w.starts_with('"') && w.ends_with('"')) || 
                 (w.starts_with('\'') && w.ends_with('\'')) => {
                Style {
                    foreground: Color { r: 0x98, g: 0xC3, b: 0x79, a: 255 }, // Green
                    background: Color { r: 0x27, g: 0x28, b: 0x22, a: 255 },
                    font_style: FontStyle::empty(),
                }
            }
            // Check for comments
            w if w.starts_with('#') => {
                Style {
                    foreground: Color { r: 0x75, g: 0x71, b: 0x5E, a: 255 }, // Gray
                    background: Color { r: 0x27, g: 0x28, b: 0x22, a: 255 },
                    font_style: FontStyle::ITALIC,
                }
            }
            // Check for numbers
            w if w.chars().all(|c| c.is_numeric() || c == '.') => {
                Style {
                    foreground: Color { r: 0xAE, g: 0x81, b: 0xFF, a: 255 }, // Purple
                    background: Color { r: 0x27, g: 0x28, b: 0x22, a: 255 },
                    font_style: FontStyle::empty(),
                }
            }
            _ => self.default_style(),
        }
    }

    /// Default text style
    fn default_style(&self) -> Style {
        Style {
            foreground: Color { r: 0xF8, g: 0xF8, b: 0xF2, a: 255 }, // White
            background: Color { r: 0x27, g: 0x28, b: 0x22, a: 255 }, // Dark background
            font_style: FontStyle::empty(),
        }
    }

    /// Get current Neovim colorscheme colors for tree-sitter highlights
    pub fn get_current_theme_colors(&self) -> Result<HashMap<String, Color>, Error> {
        // For now, return a basic color scheme
        // In a full implementation, this would query Neovim's highlight groups
        let mut colors = HashMap::new();
        
        colors.insert("keyword".to_string(), Color { r: 0x79, g: 0x79, b: 0xFF, a: 255 });
        colors.insert("string".to_string(), Color { r: 0x98, g: 0xC3, b: 0x79, a: 255 });
        colors.insert("comment".to_string(), Color { r: 0x75, g: 0x71, b: 0x5E, a: 255 });
        colors.insert("number".to_string(), Color { r: 0xAE, g: 0x81, b: 0xFF, a: 255 });
        
        Ok(colors)
    }
}