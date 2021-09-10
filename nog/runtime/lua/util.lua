function nog.clone(value, is_deep)
  local t = type(value)

  if t == "string" then
    local len = #value
    local res = ""
    local i = 1

    while i <= len do
      res = res .. string.char(value:byte(len))
      len = len - 1
    end

    return res
  end

  error("Unsupported type: " .. t)
end

function nog.tbl_filter(tbl, f)
  local res = {}
  for _, x in ipairs(tbl) do
    if f(x) then
      table.insert(res, x)
    end
  end
  return res
end

function nog.tbl_map(tbl, f)
  local res = {}
  for _, x in ipairs(tbl) do
    table.insert(res, f(x))
  end
  return res
end

function nog.split(s, sep)
  if sep == nil then
    sep = "%s"
  end
  local t={}
  for str in string.gmatch(s, "([^"..sep.."]+)") do
    table.insert(t, str)
  end
  return t
end
