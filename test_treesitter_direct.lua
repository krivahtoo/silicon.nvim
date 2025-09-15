-- Test tree-sitter functionality directly

local silicon = require('silicon')

-- Test the capture function with tree-sitter enabled
print("Testing tree-sitter capture...")

-- Set filetype for current buffer
vim.bo.filetype = 'python'

-- Add some sample code to current buffer
local code = {
    'def hello(name):',
    '    """Say hello to someone."""',
    '    return f"Hello, {name}!"',
    '',
    '# Test the function',
    'print(hello("World"))'
}

vim.api.nvim_buf_set_lines(0, 0, -1, false, code)

-- Test with tree-sitter enabled
local ok1, result1 = pcall(silicon.capture, {
    start = 1,
    ['end'] = 6,
    use_treesitter = true,
    output = {
        clipboard = false,
        path = '/tmp',
        format = 'test_ts.png'
    }
})

if not ok1 then
    print("Tree-sitter capture failed: " .. tostring(result1))
else
    print("Tree-sitter capture succeeded!")
end

-- Test with syntect (original)
local ok2, result2 = pcall(silicon.capture, {
    start = 1,
    ['end'] = 6,
    use_treesitter = false,
    output = {
        clipboard = false,
        path = '/tmp', 
        format = 'test_syntect.png'
    }
})

if not ok2 then
    print("Syntect capture failed: " .. tostring(result2))
else
    print("Syntect capture succeeded!")
end