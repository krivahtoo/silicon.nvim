<div align="center">

# silicon.nvim

Render beautiful image of your code in neovim using [silicon](https://github.com/Aloxaf/silicon).

[![Release](https://github.com/krivahtoo/silicon.nvim/actions/workflows/release.yml/badge.svg)](https://github.com/krivahtoo/silicon.nvim/actions/workflows/ci.yml)
![License](https://img.shields.io/github/license/krivahtoo/silicon.nvim)
![Neovim version](https://img.shields.io/badge/Neovim-0.7-57A143?logo=neovim)

</div>



https://user-images.githubusercontent.com/41364823/208242473-a65d0a0e-37a9-47f1-a4c2-fdc4da2f50ea.mp4



## Installation

### Requirements

- nvim `v0.7`
- **[Optional]** cargo and rust toolchain

#### Packer

```lua
use {'krivahtoo/silicon.nvim', run = './install.sh'}
```

Build from source (requires cargo)

```lua
use {'krivahtoo/silicon.nvim', run = './install.sh build'}
```

#### Vim-Plug

```vim
Plug 'krivahtoo/silicon.nvim', { 'do': './install.sh' }
```

Build from source (requires cargo)

```vim
Plug 'krivahtoo/silicon.nvim', { 'do': './install.sh build' }
```

## Configuration

Initialize the plugin.

Lua init file:
```lua
require('silicon').setup({
  font = 'FantasqueSansMono Nerd Font=16',
  theme = 'Monokai Extended',
})
```

Vimscript init file:
```vim
lua << EOF
require('silicon').setup({
  font = 'FantasqueSansMono Nerd Font=16',
  theme = 'Monokai Extended',
})
EOF
```

The `setup` function accepts the following table:

```lua
{
  -- Output configuration for the saved image
  output = {
    -- (string) The full path of the file to save to.
    file = "",
    -- (boolean) Whether to copy the image to clipboard instead of saving to file.
    clipboard = true,
    -- (string) Where to save images, defaults to the current directory.
    --  e.g. /home/user/Pictures
    path = ".",
    -- (string) The filename format to use. Can include placeholders for date and time.
    -- https://time-rs.github.io/book/api/format-description.html#components
    format = "silicon_[year][month][day]_[hour][minute][second].png",
  },

  -- Font and theme configuration for the screenshot.
  font = 'Hack=20', -- (string) The font and font size to use for the screenshot.
  -- (string) The color theme to use for syntax highlighting.
  -- It can be a theme name or path to a .tmTheme file.
  theme = 'Dracula',

  -- Background and shadow configuration for the screenshot
  background = '#eff', -- (string) The background color for the screenshot.
  shadow = {
    blur_radius = 0.0, -- (number) The blur radius for the shadow, set to 0.0 for no shadow.
    offset_x = 0, -- (number) The horizontal offset for the shadow.
    offset_y = 0, -- (number) The vertical offset for the shadow.
    color = '#555' -- (string) The color for the shadow.
  },

  pad_horiz = 100, -- (number) The horizontal padding.
  pad_vert = 80, -- (number) The vertical padding.
  line_number = false, -- (boolean) Whether to show line numbers in the screenshot.
  line_pad = 2, -- (number) The padding between lines.
  line_offset = 1, -- (number) The starting line number for the screenshot.
  tab_width = 4, -- (number) The tab width for the screenshot.
  gobble = false, -- (boolean) Whether to trim extra indentation.
  highlight_selection = false, -- (boolean) Whether to capture the whole file and highlight selected lines.
  round_corner = true,
  window_controls = true, -- (boolean) Whether to show window controls (minimize, maximize, close) in the screenshot.
  window_title = nil, -- (function) A function that returns the window title as a string.

  -- Watermark configuration for the screenshot
  watermark = {
    text = nil, -- (string) The text to use as the watermark, set to nil to disable.
    color = '#222', -- (string) The color for the watermark text.
    -- (string) The style for the watermark text, possible values are:
    -- 'bold', 'italic', 'bolditalic', or anything else defaults to 'regular'.
    style = 'bold',
  },
}
```

### Example

![image](https://user-images.githubusercontent.com/41364823/219902305-6efa37cf-4ee4-4e6b-803b-39c344a56dfe.png)

## Usage

Command:

```bash
:'<,'>Silicon[!] [file]
# Defaults to clipboard if [file] is not specified.
# With bang file is saved to specified output.file in the
# output.format specified
# Also mapped to 'SS' in Visual mode
```