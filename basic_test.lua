-- Basic test just to check plugin loading

local ok, silicon = pcall(require, 'silicon')
if not ok then
    print("Failed to load silicon: " .. tostring(silicon))
    return
end

print("Silicon plugin loaded successfully")
print("Available functions:")
for k, v in pairs(silicon) do
    print("  " .. k .. " : " .. type(v))
end

-- Test basic functions that should work
print("Version: " .. silicon.version)

local ok2, themes = pcall(silicon.list_themes)
if ok2 then
    print("Available themes: " .. #themes)
else
    print("Failed to list themes: " .. tostring(themes))
end