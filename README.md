<div align="center">

# silicon.nvim

Render beautiful image of your code in neovim using [silicon](https://github.com/Aloxaf/silicon).

[![Release](https://github.com/krivahtoo/silicon.nvim/actions/workflows/release.yml/badge.svg)](https://github.com/krivahtoo/silicon.nvim/actions/workflows/ci.yml)
![License](https://img.shields.io/github/license/krivahtoo/silicon.nvim)
![Neovim version](https://img.shields.io/badge/Neovim-0.7-57A143?logo=neovim)

</div>

![image](https://user-images.githubusercontent.com/41364823/194313504-35f02cff-1e58-45b1-8951-eed8d172b622.png)

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
```
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
  -- The following key is required if you want to save image to file instead of clipboard
  output = string
  -- The following keys are all optional
  -- with default values
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
  round_corner = true,
  window_controls = true,
}
```

## Usage

Command:

```bash
:'<,'>Silicon [file]
# Defaults to clipboard if [file] is not specified.
# Also mapped to 'SS' in Visual mode
```
