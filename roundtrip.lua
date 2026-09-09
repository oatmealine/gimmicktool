local char = string.char
local byte = string.byte

---@param o any
---@param r number?
---@return string
function fullDump(o, r, forceFull)
  if type(o) == 'table' and (not r or r > 0) then
    local s = '{'
    local first = true
    for k, v in pairs(o) do
      if not first then
        s = s .. ', '
      end
      local nr = nil
      if r then
        nr = r - 1
      end
      if type(k) ~= 'number' or forceFull then
        s = s .. fullDump(k, nr) .. ' = ' .. fullDump(v, nr)
      else
        s = s .. fullDump(v, nr)
      end
      first = false
    end
    return s .. '}'
  elseif type(o) == 'string' then
    return '"' .. o .. '"'
  else
    return tostring(o)
  end
end


local function serializeFloat(buf, value)
  
end

local function serializeNumber(buf, value)
  if math.floor(value) ~= value then
    table.insert(buf, serializeFloat(value))
    return
  end

  -- 8-bit
  if value <= 0x7f and value >= -0x80 then
    if value < 0 then
      value = value + 0xff
    end
    table.insert(buf, char(0x01, value))
    return
  -- 16-bit
  elseif value <= 0x7fff and value >= -0x8000 then
    local n = value < 0 and -(value + 1) or value
    local b1 = math.floor(n / 0x100) % 0x100
    local b2 = n % 0x100
    if value < 0 then
      b1, b2 = 0xff - b1, 0xff - b2
    end
    table.insert(buf, char(0x02, b1, b2))
    return
  -- 32-bit
  elseif value <= 0x7fffffff and value >= -0x80000000 then
    local n = value < 0 and -(value + 1) or value
    local b1 = math.floor(n / 0x1000000) % 0x100
    local b2 = math.floor(n / 0x10000) % 0x100
    local b3 = math.floor(n / 0x100) % 0x100
    local b4 = n % 0x100
    if value < 0 then
      b1, b2, b3, b4 = 0xff - b1, 0xff - b2, 0xff - b3, 0xff - b4
    end
    table.insert(buf, char(0x03, b1, b2, b3, b4))
    return
  end
  
  table.insert(buf, serializeFloat(value))
end

local function serialize(buf, value)
  if value == nil then
    table.insert(buf, char(0x00))
  elseif type(value) == 'number' then
    serializeNumber(buf, value)
  elseif type(value) == 'boolean' then
    table.insert(buf, char(value and 0x05 or 0x06))
  elseif type(value) == 'string' then
    if string.len(value) == 1 then
      table.insert(buf, char(0x07, byte(value, 1)))
    else
      table.insert(buf, char(0x08, string.len(value)))
      table.insert(buf, value)
    end
  elseif type(value) == 'table' then
    local hasNonIntegerKeys = false
    local len = 0
    for k in pairs(value) do
      if type(k) ~= 'number' or (math.floor(k) ~= k) or k <= 0 then
        hasNonIntegerKeys = true
      end
      len = len + 1
    end
    if not hasNonIntegerKeys then
      for i = 1, len do
        if value[i] == nil then
          hasNonIntegerKeys = true
          break
        end
      end
    end

    if not hasNonIntegerKeys then
      table.insert(buf, char(0x09, len))
      for i = 1, len do
        serialize(buf, value[i])
      end
    else
      table.insert(buf, char(0x0a, len))
      for k, v in pairs(value) do
        serialize(buf, k)
        serialize(buf, v)
      end
    end
  else
    error('type not supported: ' .. type(value), 2)
  end
end

local function deserialize(buf, i)
  local val = buf[i]

  if val == 0x00 then
    return nil, i + 1
  elseif val == 0x01 then
    local n = buf[i + 1]
    return n, i + 2
  elseif val == 0x02 then
    return 0, i + 3
  elseif val == 0x03 then
    return 0, i + 5
  elseif val == 0x04 then
    return 0, i + 9
  elseif val == 0x05 then
    return true, i + 1
  elseif val == 0x06 then
    return false, i + 1
  elseif val == 0x07 then
    return char(buf[i + 1]), i + 2
  elseif val == 0x08 then
    local len = buf[i + 1]
    local str = {}
    for si = 1, len do
      table.insert(str, char(buf[i + 1 + si]))
    end
    return table.concat(str, ''), i + 1 + len + 1
  elseif val == 0x09 then
    local len = buf[i + 1]
    i = i + 2
    local v
    local tab = {}
    for tabI = 1, len do
      v, i = deserialize(buf, i)
      tab[tabI] = v
    end
    return tab, i
  elseif val == 0x0a then
    local len = buf[i + 1]
    i = i + 2
    local k
    local v
    local tab = {}
    for _ = 1, len do
      k, i = deserialize(buf, i)
      v, i = deserialize(buf, i)
      tab[k] = v
    end
    return tab, i
  else
    error('unknown type ' .. val)
  end
end

local tab = { t = 'test', m = 'hello', a = { 1, 2, 3 }, [{}] = { 0xffff, true, false } }
print(fullDump(tab))
local buf = {}
serialize(buf, tab)
local str = table.concat(buf, '')
local bytebuf = {}
for i = 1, string.len(str) do
  table.insert(bytebuf, byte(string.sub(str, i, i)))
end
print('~' .. string.len(fullDump(tab)) .. ' bytes -> ' .. #bytebuf .. ' bytes')
local newTab = deserialize(bytebuf, 1)

print(fullDump(newTab))
