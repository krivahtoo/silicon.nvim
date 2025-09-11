-- Minimal test to isolate segfault

local silicon = require('silicon')

print("Testing minimal capture...")

-- Set up buffer with minimal content
vim.bo.filetype = 'python'
vim.api.nvim_buf_set_lines(0, 0, -1, false, {'print("hello")'})

-- Test only syntect (original) first
local ok, result = pcall(silicon.capture, {
    start = 1,
    ['end'] = 1,
    use_treesitter = false,
    output = {
        clipboard = false,
        path = '/tmp',
        format = 'minimal_test.png'
    }
})

if not ok then
    print("Syntect capture failed: " .. tostring(result))
else
    print("Syntect capture succeeded!")
end