-- Debug script for silicon.nvim

-- Try to load the plugin with error handling
local ok, silicon = pcall(require, 'silicon')
if not ok then
    print("Failed to load silicon plugin: " .. tostring(silicon))
    return
end

print("Successfully loaded silicon plugin")
print("Available functions:")
for k, v in pairs(silicon) do
    print("  " .. k .. " : " .. type(v))
end

-- Test setup with minimal config
local ok2, result = pcall(function()
    return silicon.setup({
        use_treesitter = false
    })
end)

if not ok2 then
    print("Setup failed: " .. tostring(result))
else
    print("Setup successful!")
end