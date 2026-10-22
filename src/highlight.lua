-- Collects tree-sitter highlights for a range of lines and resolves them
-- against the current colorscheme. Evaluated from Rust via `luaeval`.
--
-- args: { bufnr = number, start = number (0-based), stop = number (exclusive), gobble = boolean }
-- returns nil when the buffer has no tree-sitter parser, otherwise:
-- { normal = { fg, bg }, lines = { { { text, fg, bold, italic, underline }, ... }, ... } }
return function(args)
  local bufnr, start, stop = args.bufnr, args.start, args.stop

  local ok, parser = pcall(vim.treesitter.get_parser, bufnr)
  if not ok or not parser then
    return nil
  end
  parser:parse({ start, stop })

  local function hex(n)
    return n and string.format('#%06x', n) or nil
  end

  local normal = vim.api.nvim_get_hl(0, { name = 'Normal', link = false })
  local dark = vim.o.background == 'dark'
  local normal_fg = hex(normal.fg) or (dark and '#e0e2ea' or '#14161b')
  local normal_bg = hex(normal.bg) or (dark and '#14161b' or '#e0e2ea')

  -- Resolve `@capture.name.lang`, falling back to `@capture.name`, `@capture`
  -- the same way Neovim does when a more specific group is not defined.
  local cache = {}
  local function resolve(group)
    if cache[group] == nil then
      local name, hl = group, {}
      while true do
        hl = vim.api.nvim_get_hl(0, { name = name, link = false })
        if next(hl) ~= nil then
          break
        end
        local parent = name:match('^(.+)%.[^.]+$')
        if not parent then
          break
        end
        name = parent
      end
      cache[group] = {
        fg = hex(hl.fg),
        bold = hl.bold,
        italic = hl.italic,
        underline = hl.underline or hl.undercurl,
      }
    end
    return cache[group]
  end

  local captures = {}
  parser:for_each_tree(function(tree, ltree)
    local lang = ltree:lang()
    local query = vim.treesitter.query.get(lang, 'highlights')
    if not query then
      return
    end
    for id, node, metadata in query:iter_captures(tree:root(), bufnr, start, stop) do
      local name = query.captures[id]
      if name and name:sub(1, 1) ~= '_' and name ~= 'spell' and name ~= 'nospell' then
        local sr, sc, er, ec = node:range()
        local priority = tonumber(metadata.priority or (metadata[id] and metadata[id].priority)) or 100
        captures[#captures + 1] = {
          sr = sr, sc = sc, er = er, ec = ec,
          hl = resolve('@' .. name .. '.' .. lang),
          priority = priority,
          order = #captures,
        }
      end
    end
  end)

  -- Lower priority first; equal priority keeps query order so later captures win.
  table.sort(captures, function(a, b)
    if a.priority ~= b.priority then
      return a.priority < b.priority
    end
    return a.order < b.order
  end)

  local lines = vim.api.nvim_buf_get_lines(bufnr, start, stop, false)

  -- Per-byte attributes for every line.
  local attrs = {}
  for i, line in ipairs(lines) do
    local row = {}
    for b = 1, #line do
      row[b] = {}
    end
    attrs[i] = row
  end

  for _, c in ipairs(captures) do
    for r = math.max(c.sr, start), math.min(c.er, stop - 1) do
      local row = attrs[r - start + 1]
      local from = r == c.sr and c.sc + 1 or 1
      local to = r == c.er and c.ec or #row
      for b = from, math.min(to, #row) do
        local a = row[b]
        -- Merge like Neovim does: only attributes the group sets override.
        if c.hl.fg then a.fg = c.hl.fg end
        if c.hl.bold ~= nil then a.bold = c.hl.bold end
        if c.hl.italic ~= nil then a.italic = c.hl.italic end
        if c.hl.underline ~= nil then a.underline = c.hl.underline end
      end
    end
  end

  -- Same rule as `utils::get_lines`: strip the first line's leading spaces from every line.
  local gobble = 0
  if args.gobble and lines[1] then
    local first = lines[1]:find('[^ ]')
    gobble = first and first - 1 or 0
  end

  local out = {}
  for i, line in ipairs(lines) do
    local skip = 0
    if gobble > 0 then
      local idx = vim.fn.byteidx(line, gobble)
      skip = idx < 0 and #line or idx
    end

    local spans = {}
    local row = attrs[i]
    local b = skip + 1
    while b <= #line do
      local a = row[b]
      local fg, bold, italic, ul = a.fg or normal_fg, a.bold or false, a.italic or false, a.underline or false
      local e = b
      while e < #line do
        local n = row[e + 1]
        if (n.fg or normal_fg) ~= fg or (n.bold or false) ~= bold or (n.italic or false) ~= italic
          or (n.underline or false) ~= ul then
          break
        end
        e = e + 1
      end
      spans[#spans + 1] = { text = line:sub(b, e), fg = fg, bold = bold, italic = italic, underline = ul }
      b = e + 1
    end
    -- Never return an empty table: it would not round-trip as a list.
    if #spans == 0 then
      spans[1] = { text = '', fg = normal_fg, bold = false, italic = false, underline = false }
    end
    out[i] = spans
  end

  return { normal = { fg = normal_fg, bg = normal_bg }, lines = out }
end
