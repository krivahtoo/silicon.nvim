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
  -- The following keys are all optional
  -- with default values
  -- The following key is required if you want to save image to file instead of clipboard
  output = {
    file = "", -- full path of the file to save to.
    clipboard = true,
    path = ".", -- where to save images e.g. /home/user/Pictures
    format = "silicon_[year][month][day]_[hour][minute][second].png",
  },
  font = 'Hack=20',
  theme = 'Dracula',
  background = '#eff',
  shadow = {
    blur_radius = 0.0,
    offset_x = 0,
    offset_y = 0,
    color = '#555'
  },
  pad_horiz = 100,
  pad_vert = 80,
  line_number = false,
  line_pad = 2,
  line_offset = 1,
  tab_width = 4,
  gobble = false, -- trim extra identation.
  round_corner = true,
  window_controls = true,
  window_title = nil, -- a function returning window title as a string
  watermark = {
    text = nil, -- add this to enable watermark on the bottom-right.
    color = '#222',
    style = 'bold', -- possible values: 'bold' | 'italic' | 'bolditalic' | anything else defaults to 'regular'.
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