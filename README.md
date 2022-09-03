# silicon.nvim

Render beautiful image of your code in neovim using [silicon](https://github.com/Aloxaf/silicon).

## Installation

### Requirements

- nvim `v0.7`
- cargo

#### Packer

```lua
use {'krivahtoo/silicon.nvim', run = './install.sh'}
```

#### Vim-Plug

```vim
Plug 'krivahtoo/silicon.nvim', { 'do': './install.sh' }
```

## Usage

```lua
require('silicon').setup({
  line1 = 0, -- required
  line2 = 0, -- required
  font = 'FantasqueSansMono Nerd Font',
  theme = 'Monokai Extended',
})
```

Command:

```bash
Silicon [file]
```


The `capture` function accepts the following table:

```lua
{
  line1 = number, -- alias to 'start'
  line2 = number, -- alias to 'end'
  -- The following keys are all optional
  -- with default values
  font = 'Hack',
  font_size = 20.0,
  theme = 'Dracula',
  line_number = true,
  line_pad = 2,
  line_offset = 1,
  round_corner = true,
  window_controls = true,
  -- The following key is required if you want to save image to file instead of clipboard
  output = string
}
```

