-- Test script for silicon.nvim tree-sitter support

-- Load the plugin
local silicon = require('silicon')

-- Test 1: Setup with syntect (original behavior)
print("Testing setup with syntect highlighting...")
silicon.setup({
    font = 'Hack=16',
    theme = 'Dracula',
    use_treesitter = false,
    output = {
        clipboard = false,
        path = '/tmp',
        format = 'test_syntect.png'
    }
})

-- Test 2: Setup with tree-sitter
print("Testing setup with tree-sitter highlighting...")
silicon.setup({
    font = 'Hack=16',
    theme = 'Dracula',
    use_treesitter = true,
    output = {
        clipboard = false,
        path = '/tmp',
        format = 'test_treesitter.png'
    }
})

print("Plugin setup successful!")
print("Available functions:")
for k, v in pairs(silicon) do
    print("  " .. k .. " : " .. type(v))
end