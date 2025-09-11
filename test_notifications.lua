-- Test tree-sitter notifications

local silicon = require('silicon')

print("Testing tree-sitter notifications...")

-- Setup with tree-sitter
silicon.setup({
    use_treesitter = true,
    theme = 'Dracula'
})

-- Set up buffer content for testing
vim.bo.filetype = 'python'
vim.api.nvim_buf_set_lines(0, 0, -1, false, {
    'def hello():',
    '    print("Hello from tree-sitter!")',
    '    return True'
})

-- Test the capture function with tree-sitter 
print("Testing capture with tree-sitter notifications...")
local ok, result = pcall(silicon.capture, {
    start = 1,
    ['end'] = 3,
    use_treesitter = true,
    theme = 'Dracula',
    output = {
        clipboard = false,
        path = '/tmp',
        format = 'treesitter_test.png'
    }
})

if ok then
    print("Tree-sitter capture initiated successfully!")
else
    print("Tree-sitter capture failed: " .. tostring(result))
end