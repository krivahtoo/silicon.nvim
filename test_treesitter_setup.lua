-- Test tree-sitter setup

local silicon = require('silicon')

print("Testing tree-sitter setup...")

-- Test 1: Setup with syntect (original behavior)
local ok1, result1 = pcall(silicon.setup, {
    font = 'Hack=16',
    theme = 'Dracula',
    use_treesitter = false,
    output = {
        clipboard = false,
        path = '/tmp',
    }
})

if not ok1 then
    print("Syntect setup failed: " .. tostring(result1))
else
    print("Syntect setup succeeded!")
end

-- Test 2: Setup with tree-sitter 
local ok2, result2 = pcall(silicon.setup, {
    font = 'Hack=16',
    theme = 'Dracula',
    use_treesitter = true,
    output = {
        clipboard = false,
        path = '/tmp',
    }
})

if not ok2 then
    print("Tree-sitter setup failed: " .. tostring(result2))
else
    print("Tree-sitter setup succeeded!")
end

print("Setup tests completed!")