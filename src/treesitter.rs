use crate::error::Error;
use std::collections::HashMap;
use syntect::highlighting::{Color, FontStyle, Style};
use nvim_oxi::api::get_hl_by_name;

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
        // Get current theme colors from Neovim first
        let theme_colors = self.get_current_theme_colors()?;
        
        // For now, use enhanced highlighting with Neovim theme colors
        // This integrates with Neovim's color scheme while we work on full tree-sitter parsing
        self.enhanced_highlight_with_theme(code, filetype, &theme_colors)
    }

    /// Enhanced highlighting using Neovim's current theme colors
    fn enhanced_highlight_with_theme(
        &self,
        code: &str,
        _filetype: &str,
        theme_colors: &HashMap<String, Color>,
    ) -> Result<Vec<Vec<(Style, String)>>, Error> {
        let lines: Vec<&str> = code.lines().collect();
        let mut result = Vec::new();

        for line in lines {
            let mut line_result = Vec::new();
            
            // Simple keyword-based highlighting using Neovim colors
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut current_pos = 0;
            
            for word in words {
                // Find the position of this word in the line
                if let Some(word_pos) = line[current_pos..].find(word) {
                    let actual_pos = current_pos + word_pos;
                    
                    // Add any whitespace before the word
                    if actual_pos > current_pos {
                        let whitespace = &line[current_pos..actual_pos];
                        line_result.push((self.get_themed_style("variable", theme_colors), whitespace.to_string()));
                    }
                    
                    // Add the word with appropriate highlighting using theme colors
                    let style = self.get_themed_word_style(word, theme_colors);
                    line_result.push((style, word.to_string()));
                    
                    current_pos = actual_pos + word.len();
                }
            }
            
            // Add any remaining text
            if current_pos < line.len() {
                let remaining = &line[current_pos..];
                line_result.push((self.get_themed_style("variable", theme_colors), remaining.to_string()));
            }
            
            // If no words were found, add the entire line
            if line_result.is_empty() {
                line_result.push((self.get_themed_style("variable", theme_colors), line.to_string()));
            }
            
            result.push(line_result);
        }

        Ok(result)
    }

    /// Get style for a specific word using theme colors
    fn get_themed_word_style(&self, word: &str, theme_colors: &HashMap<String, Color>) -> Style {
        let (style_name, font_style) = match word {
            // Python keywords
            "def" | "class" | "import" | "from" | "if" | "else" | "elif" | "for" | "while" | 
            "try" | "except" | "finally" | "with" | "as" | "return" | "yield" | "break" | 
            "continue" | "pass" | "raise" | "assert" | "global" | "nonlocal" | "lambda" |
            "and" | "or" | "not" | "in" | "is" | "True" | "False" | "None" => {
                ("keyword", FontStyle::BOLD)
            }
            // Rust keywords
            "fn" | "let" | "mut" | "const" | "static" | "struct" | "enum" | "trait" | 
            "impl" | "mod" | "pub" | "use" | "crate" | "self" | "super" | "match" |
            "loop" => {
                ("keyword", FontStyle::BOLD)
            }
            // JavaScript keywords  
            "function" | "var" | "async" | "await" | "typeof" | "instanceof" |
            "extends" | "export" | "default" => {
                ("keyword", FontStyle::BOLD)
            }
            // Check for string literals
            w if (w.starts_with('"') && w.ends_with('"')) || 
                 (w.starts_with('\'') && w.ends_with('\'')) => {
                ("string", FontStyle::empty())
            }
            // Check for comments
            w if w.starts_with('#') || w.starts_with("//") || w.starts_with("/*") => {
                ("comment", FontStyle::ITALIC)
            }
            // Check for numbers
            w if w.chars().all(|c| c.is_numeric() || c == '.' || c == '_') && 
                 w.chars().any(|c| c.is_numeric()) => {
                ("number", FontStyle::empty())
            }
            // Check for function calls (words followed by parentheses - simplified)
            w if w.ends_with('(') => {
                ("function", FontStyle::empty())
            }
            _ => ("variable", FontStyle::empty()),
        };

        self.get_themed_style(style_name, theme_colors).with_font_style(font_style)
    }

    /// Get themed style for a specific highlight group
    fn get_themed_style(&self, group_name: &str, theme_colors: &HashMap<String, Color>) -> Style {
        let color = theme_colors.get(group_name)
            .unwrap_or(&self.get_fallback_color(group_name))
            .clone();

        Style {
            foreground: color,
            background: self.get_background_color(),
            font_style: FontStyle::empty(),
        }
    }

    /// Get background color (dark theme default)
    fn get_background_color(&self) -> Color {
        Color { r: 0x27, g: 0x28, b: 0x22, a: 255 }
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
        let mut colors = HashMap::new();
        
        // Query Neovim's highlight groups to get current theme colors
        let highlight_groups = [
            ("Keyword", "keyword"),
            ("String", "string"), 
            ("Comment", "comment"),
            ("Number", "number"),
            ("Function", "function"),
            ("Type", "type"),
            ("Constant", "constant"),
            ("Variable", "variable"),
            ("Operator", "operator"),
            ("Identifier", "identifier"),
            ("Statement", "statement"),
            ("PreProc", "preproc"),
            ("Special", "special"), 
            ("Error", "error"),
            ("Todo", "todo"),
        ];

        for (nvim_group, ts_group) in highlight_groups.iter() {
            match self.get_highlight_color(nvim_group) {
                Ok(color) => {
                    colors.insert(ts_group.to_string(), color);
                }
                Err(_) => {
                    // Use fallback color if we can't get the highlight group
                    colors.insert(ts_group.to_string(), self.get_fallback_color(ts_group));
                }
            }
        }

        // Also try to get tree-sitter specific groups (if they exist)
        let ts_groups = [
            "@keyword",
            "@string", 
            "@comment",
            "@number",
            "@function",
            "@type",
            "@constant",
            "@variable",
            "@operator",
        ];

        for group in ts_groups.iter() {
            if let Ok(color) = self.get_highlight_color(group) {
                let clean_name = group.trim_start_matches('@');
                colors.insert(clean_name.to_string(), color);
            }
        }

        Ok(colors)
    }

    /// Get color for a specific Neovim highlight group
    fn get_highlight_color(&self, group_name: &str) -> Result<Color, Error> {
        // Try to get highlight group by name
        match get_hl_by_name(group_name, true) {
            Ok(hl_attrs) => {
                // Extract foreground color from highlight attributes  
                if let Some(fg) = hl_attrs.foreground {
                    let r = ((fg >> 16) & 0xFF) as u8;
                    let g = ((fg >> 8) & 0xFF) as u8;
                    let b = (fg & 0xFF) as u8;
                    return Ok(Color { r, g, b, a: 255 });
                }
                
                // Try ctermfg as fallback
                if let Some(ctermfg) = hl_attrs.fg_indexed {
                    return Ok(self.cterm_to_rgb(ctermfg as u8));
                }
                
                Err(Error::Generic(format!("No color found for highlight group: {}", group_name)))
            }
            Err(e) => {
                // For debugging - don't fail completely, just use fallback
                Err(Error::Generic(format!("Failed to get highlight group {}: {}", group_name, e)))
            }
        }
    }

    /// Convert cterm color to RGB
    fn cterm_to_rgb(&self, cterm: u8) -> Color {
        // Basic cterm color mapping (16 colors)
        match cterm {
            0 => Color { r: 0x00, g: 0x00, b: 0x00, a: 255 }, // Black
            1 => Color { r: 0x80, g: 0x00, b: 0x00, a: 255 }, // Dark Red
            2 => Color { r: 0x00, g: 0x80, b: 0x00, a: 255 }, // Dark Green
            3 => Color { r: 0x80, g: 0x80, b: 0x00, a: 255 }, // Dark Yellow
            4 => Color { r: 0x00, g: 0x00, b: 0x80, a: 255 }, // Dark Blue
            5 => Color { r: 0x80, g: 0x00, b: 0x80, a: 255 }, // Dark Magenta
            6 => Color { r: 0x00, g: 0x80, b: 0x80, a: 255 }, // Dark Cyan
            7 => Color { r: 0xC0, g: 0xC0, b: 0xC0, a: 255 }, // Light Gray
            8 => Color { r: 0x80, g: 0x80, b: 0x80, a: 255 }, // Dark Gray
            9 => Color { r: 0xFF, g: 0x00, b: 0x00, a: 255 }, // Red
            10 => Color { r: 0x00, g: 0xFF, b: 0x00, a: 255 }, // Green
            11 => Color { r: 0xFF, g: 0xFF, b: 0x00, a: 255 }, // Yellow
            12 => Color { r: 0x00, g: 0x00, b: 0xFF, a: 255 }, // Blue
            13 => Color { r: 0xFF, g: 0x00, b: 0xFF, a: 255 }, // Magenta
            14 => Color { r: 0x00, g: 0xFF, b: 0xFF, a: 255 }, // Cyan
            15 => Color { r: 0xFF, g: 0xFF, b: 0xFF, a: 255 }, // White
            _ => Color { r: 0xF8, g: 0xF8, b: 0xF2, a: 255 }, // Default
        }
    }

    /// Get fallback color for a tree-sitter group
    fn get_fallback_color(&self, ts_group: &str) -> Color {
        match ts_group {
            "keyword" => Color { r: 0x79, g: 0x79, b: 0xFF, a: 255 }, // Blue
            "string" => Color { r: 0x98, g: 0xC3, b: 0x79, a: 255 }, // Green
            "comment" => Color { r: 0x75, g: 0x71, b: 0x5E, a: 255 }, // Gray
            "number" => Color { r: 0xAE, g: 0x81, b: 0xFF, a: 255 }, // Purple
            "function" => Color { r: 0x66, g: 0xD9, b: 0xEF, a: 255 }, // Light Blue
            "type" => Color { r: 0x66, g: 0xD9, b: 0xEF, a: 255 }, // Light Blue
            "constant" => Color { r: 0xAE, g: 0x81, b: 0xFF, a: 255 }, // Purple
            "variable" => Color { r: 0xF8, g: 0xF8, b: 0xF2, a: 255 }, // White
            "operator" => Color { r: 0xF9, g: 0x26, b: 0x72, a: 255 }, // Pink
            _ => Color { r: 0xF8, g: 0xF8, b: 0xF2, a: 255 }, // Default white
        }
    }
}

/// Extension trait to add font style to existing Style
trait StyleExt {
    fn with_font_style(self, font_style: FontStyle) -> Self;
}

impl StyleExt for Style {
    fn with_font_style(mut self, font_style: FontStyle) -> Self {
        self.font_style = font_style;
        self
    }
}