use crate::error::Error;
use std::collections::HashMap;
use syntect::highlighting::{Color, FontStyle, Style};
use nvim_oxi::api::{get_hl_by_name, get_runtime_file, eval};
use tree_sitter_highlight::{Highlighter, HighlightConfiguration, HighlightEvent};
use tree_sitter::Language;

// Conditional static language support (only if builtin-parsers feature is enabled)
#[cfg(feature = "tree-sitter-lua")]
extern "C" { fn tree_sitter_lua() -> Language; }
#[cfg(feature = "tree-sitter-rust")]
extern "C" { fn tree_sitter_rust() -> Language; }
#[cfg(feature = "tree-sitter-python")]
extern "C" { fn tree_sitter_python() -> Language; }
#[cfg(feature = "tree-sitter-javascript")]
extern "C" { fn tree_sitter_javascript() -> Language; }
#[cfg(feature = "tree-sitter-typescript")]
extern "C" { fn tree_sitter_typescript() -> Language; }
#[cfg(feature = "tree-sitter-json")]
extern "C" { fn tree_sitter_json() -> Language; }
#[cfg(feature = "tree-sitter-bash")]
extern "C" { fn tree_sitter_bash() -> Language; }
#[cfg(feature = "tree-sitter-c")]
extern "C" { fn tree_sitter_c() -> Language; }
#[cfg(feature = "tree-sitter-cpp")]
extern "C" { fn tree_sitter_cpp() -> Language; }
#[cfg(feature = "tree-sitter-go")]
extern "C" { fn tree_sitter_go() -> Language; }

// Define highlight names that we will recognize (from tree-sitter-highlight docs)
const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "embedded",
    "function",
    "function.builtin",
    "keyword",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

pub struct TreesitterHighlighter {
    highlighter: Highlighter,
    configurations: HashMap<String, HighlightConfiguration>,
}

impl TreesitterHighlighter {
    pub fn new() -> Result<Self, Error> {
        let highlighter = Highlighter::new();
        let configurations = HashMap::new();
        
        // Languages will be loaded dynamically when needed using nvim-treesitter
        Ok(Self {
            highlighter,
            configurations,
        })
    }

    /// Load parser dynamically from nvim-treesitter if available
    fn get_or_load_language(&mut self, filetype: &str) -> Result<&HighlightConfiguration, Error> {
        // Check if we already have the configuration cached
        if self.configurations.contains_key(filetype) {
            return Ok(self.configurations.get(filetype).unwrap());
        }

        // Try to load the language using nvim-treesitter
        let language = self.load_nvim_treesitter_parser(filetype)?;
        let config = Self::load_language_config(filetype, language)?;
        
        self.configurations.insert(filetype.to_string(), config);
        Ok(self.configurations.get(filetype).unwrap())
    }

    /// Load a tree-sitter parser, preferring nvim-treesitter availability check
    fn load_nvim_treesitter_parser(&self, language_name: &str) -> Result<Language, Error> {
        // Check if nvim-treesitter has the parser installed
        let nvim_treesitter_available = self.check_nvim_treesitter_parser(language_name);
        
        if nvim_treesitter_available {
            // If nvim-treesitter has it, use our static parser (if available)
            // This ensures we use the same language that the user has in their editor
            self.get_static_language(language_name).or_else(|_| {
                Err(Error::Generic(format!(
                    "nvim-treesitter has {} parser but no builtin parser available. Consider using the 'builtin-parsers' feature.", 
                    language_name
                )))
            })
        } else {
            // If not available in nvim-treesitter, try static as fallback
            self.get_static_language(language_name).or_else(|_| {
                Err(Error::Generic(format!(
                    "No parser available for {}. Install with ':TSInstall {}' or enable the 'builtin-parsers' feature.", 
                    language_name, language_name
                )))
            })
        }
    }

    /// Check if nvim-treesitter has a parser for the given language
    fn check_nvim_treesitter_parser(&self, language_name: &str) -> bool {
        let lua_code = format!(
            r#"
            local ok, ts = pcall(require, 'nvim-treesitter.parsers')
            if not ok then
                return false
            end
            
            local parser_info = ts.get_parser_configs()['{}']
            if not parser_info then
                return false
            end
            
            -- Check if the parser is installed
            local lang_ok = pcall(vim.treesitter.language.get_lang, '{}')
            return lang_ok
            "#,
            language_name, language_name
        );

        match eval::<bool>(&lua_code) {
            Ok(available) => available,
            Err(_) => false, // If we can't check, assume not available
        }
    }

    /// Get static fallback language parser (only if builtin parsers are available)
    fn get_static_language(&self, language_name: &str) -> Result<Language, Error> {
        match language_name {
            #[cfg(feature = "tree-sitter-lua")]
            "lua" => Ok(unsafe { tree_sitter_lua() }),
            #[cfg(feature = "tree-sitter-rust")]
            "rust" | "rs" => Ok(unsafe { tree_sitter_rust() }),
            #[cfg(feature = "tree-sitter-python")]
            "python" | "py" => Ok(unsafe { tree_sitter_python() }),
            #[cfg(feature = "tree-sitter-javascript")]
            "javascript" | "js" => Ok(unsafe { tree_sitter_javascript() }),
            #[cfg(feature = "tree-sitter-typescript")]
            "typescript" | "ts" => Ok(unsafe { tree_sitter_typescript() }),
            #[cfg(feature = "tree-sitter-json")]
            "json" => Ok(unsafe { tree_sitter_json() }),
            #[cfg(feature = "tree-sitter-bash")]
            "bash" | "sh" => Ok(unsafe { tree_sitter_bash() }),
            #[cfg(feature = "tree-sitter-c")]
            "c" => Ok(unsafe { tree_sitter_c() }),
            #[cfg(feature = "tree-sitter-cpp")]
            "cpp" | "cxx" => Ok(unsafe { tree_sitter_cpp() }),
            #[cfg(feature = "tree-sitter-go")]
            "go" => Ok(unsafe { tree_sitter_go() }),
            _ => Err(Error::Generic(format!("No builtin parser available for language: {}. Either enable the builtin-parsers feature or ensure nvim-treesitter has the {} parser installed.", language_name, language_name)))
        }
    }

    /// Load highlighting configuration for a language by extracting queries from Neovim
    fn load_language_config(language_name: &str, language: Language) -> Result<HighlightConfiguration, Error> {
        // Extract queries from Neovim's runtime path
        let (highlight_query, injections_query, locals_query) = Self::extract_nvim_queries(language_name)?;
        
        let mut config = HighlightConfiguration::new(
            language,
            language_name,
            &highlight_query,
            &injections_query,
            &locals_query,
        ).map_err(|e| Error::Generic(format!("Failed to create highlight config for {}: {}", language_name, e)))?;
        
        // Configure the recognized highlight names
        config.configure(HIGHLIGHT_NAMES);
        
        Ok(config)
    }

    /// Extract HIGHLIGHT_QUERY, INJECTIONS_QUERY and LOCALS_QUERY from Neovim's search path
    fn extract_nvim_queries(language_name: &str) -> Result<(String, String, String), Error> {
        let highlight_query = Self::get_nvim_query(language_name, "highlights")?;
        let injections_query = Self::get_nvim_query(language_name, "injections").unwrap_or_default();
        let locals_query = Self::get_nvim_query(language_name, "locals").unwrap_or_default();
        
        Ok((highlight_query, injections_query, locals_query))
    }

    /// Get a specific query file from Neovim's runtime path
    fn get_nvim_query(language_name: &str, query_type: &str) -> Result<String, Error> {
        let query_path = format!("queries/{}/{}.scm", language_name, query_type);
        
        // Try to get the query file from Neovim's runtime path
        let files_result = get_runtime_file(&query_path, false);
        match files_result {
            Ok(files) => {
                let files_vec: Vec<_> = files.collect();
                if let Some(file_path) = files_vec.first() {
                    std::fs::read_to_string(file_path)
                        .map_err(|e| Error::Generic(format!("Failed to read query file {}: {}", file_path.display(), e)))
                } else {
                    Err(Error::Generic(format!("No query file found for {} {}", language_name, query_type)))
                }
            }
            Err(e) => Err(Error::Generic(format!("Failed to get runtime file {}: {}", query_path, e)))
        }
    }

    /// Get tree-sitter highlights for the given code and filetype
    pub fn highlight_code(
        &mut self,
        code: &str,
        filetype: &str,
    ) -> Result<Vec<Vec<(Style, String)>>, Error> {
        // Get current theme colors from Neovim first
        let theme_colors = self.get_current_theme_colors()?;
        
        // Try to get or load the language configuration dynamically
        match self.get_or_load_language(filetype) {
            Ok(_) => {
                // Use proper tree-sitter highlighting
                self.treesitter_highlight_with_theme_by_filetype(code, filetype, &theme_colors)
            }
            Err(_) => {
                // Fallback to enhanced highlighting for unsupported languages
                self.enhanced_highlight_with_theme(code, filetype, &theme_colors)
            }
        }
    }

    /// Perform actual tree-sitter highlighting using filetype lookup
    fn treesitter_highlight_with_theme_by_filetype(
        &mut self,
        code: &str,
        filetype: &str,
        theme_colors: &HashMap<String, Color>,
    ) -> Result<Vec<Vec<(Style, String)>>, Error> {
        // Check if we have the configuration - we already verified this exists
        if !self.configurations.contains_key(filetype) {
            return Err(Error::Generic(format!("No configuration found for filetype: {}", filetype)));
        }
        
        // Clone the configuration to avoid borrowing issues
        // Since HighlightConfiguration doesn't implement Clone, we need to work around this
        // by handling the highlighting in this method
        self.treesitter_highlight_with_theme_direct(code, filetype, theme_colors)
    }

    /// Direct tree-sitter highlighting without separate config borrowing
    fn treesitter_highlight_with_theme_direct(
        &mut self,
        code: &str,
        filetype: &str,
        theme_colors: &HashMap<String, Color>,
    ) -> Result<Vec<Vec<(Style, String)>>, Error> {
        // Pre-calculate all styles to avoid borrowing issues during highlighting
        let mut style_cache = HashMap::new();
        for &name in HIGHLIGHT_NAMES {
            style_cache.insert(name, self.get_themed_style(name, theme_colors));
        }
        let default_style = self.get_themed_style("variable", theme_colors);
        
        // Get the configuration - we know it exists
        let config = self.configurations.get(filetype)
            .ok_or_else(|| Error::Generic(format!("No configuration found for filetype: {}", filetype)))?;
        
        let highlights = self.highlighter.highlight(
            config,
            code.as_bytes(),
            None,
            |_| None // No language injection for now
        ).map_err(|e| Error::Generic(format!("Tree-sitter highlighting failed: {}", e)))?;

        let mut result = Vec::new();
        let mut current_line = Vec::new();
        let mut current_style = default_style.clone();
        let mut style_stack = Vec::new();

        for event in highlights {
            match event.map_err(|e| Error::Generic(format!("Highlight event error: {}", e)))? {
                HighlightEvent::Source { start, end } => {
                    let text = &code[start..end];
                    
                    // Handle line breaks
                    for line in text.lines() {
                        current_line.push((current_style.clone(), line.to_string()));
                        
                        // Check if we're at the end of a line
                        if text.contains('\n') {
                            result.push(current_line);
                            current_line = Vec::new();
                        }
                    }
                }
                HighlightEvent::HighlightStart(style_idx) => {
                    // Push current style to stack
                    style_stack.push(current_style.clone());
                    
                    // Get the highlight name for this style index
                    if let Some(highlight_name) = HIGHLIGHT_NAMES.get(style_idx.0) {
                        current_style = style_cache.get(highlight_name)
                            .cloned()
                            .unwrap_or_else(|| default_style.clone());
                    }
                }
                HighlightEvent::HighlightEnd => {
                    // Pop style from stack
                    if let Some(previous_style) = style_stack.pop() {
                        current_style = previous_style;
                    } else {
                        current_style = default_style.clone();
                    }
                }
            }
        }

        // Add the last line if there's content
        if !current_line.is_empty() {
            result.push(current_line);
        }

        // Ensure we have at least one line
        if result.is_empty() {
            result.push(vec![(default_style, String::new())]);
        }

        Ok(result)
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