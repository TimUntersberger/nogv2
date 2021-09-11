-- TODO: update package.path
package.cpath = nog.runtime_path .. "/dll/?.dll;" .. package.cpath

function nog.execute_runtime_file(path)
  return dofile(nog.runtime_path .. "/lua/" .. path)
end

nog.execute_runtime_file("util.lua")

nog.uv = require 'luv'
nog.components = nog.execute_runtime_file("components/init.lua")
nog.inspect = nog.execute_runtime_file("inspect.lua")
nog.layout = nog.execute_runtime_file("layouts/master_slave.lua")

nog.execute_runtime_file("keybindings.lua")
-- local modes = {}
-- local previous_kbs = nil
-- local current_mode = nil

-- function nog.mode(name, cb)
--   modes[name] = cb
-- end

-- function nog.toggle_mode(name)
--   local cb = modes[name]

--   assert(cb ~= nil, string.format("Mode '%s' has not been defined yet", name))

--   if current_mode ~= nil then
--     if current_mode == name then
--       local mode_kbs = nog.get_keybindings()

--       nog.__unbind_batch(nog.tbl_filter(mode_kbs, function(kb)
--         return kb.mode == "n"
--       end))

--       nog.__bind_batch(nog.tbl_filter(previous_kbs, function(kb)
--         return kb.mode == "n"
--       end))

--       current_mode = nil
--     else
--     end
--   else
--     previous_kbs = nog.get_keybindings()

--     nog.__unbind_batch(nog.tbl_filter(previous_kbs, function(kb)
--       return kb.mode == "n"
--     end))

--     cb()

--     current_mode = name
--   end
-- end

-- local function create_bind_tbl_fn(mode)
--   return function(modifier, cb, tbl)
--     for key, val in pairs(tbl) do
--       local key = string.format("%s+%s", modifier, key)
--       nog[mode .. "bind"](key, function()
--         cb(val)
--       end)
--     end
--   end
-- end

-- local function create_bind_fn(mode)
--   return function(key, cb)
--     nog.bind(mode, key, cb)
--   end
-- end

-- nog.bind = function(m, k, f)
--   table.insert(nog.__callbacks, f)
--   nog.__bind(m, k, #nog.__callbacks)
-- end

-- nog.nbind = create_bind_fn("n")
-- nog.nbind_tbl = create_bind_tbl_fn("n")

-- nog.gbind = create_bind_fn("g")
-- nog.gbind_tbl = create_bind_tbl_fn("g")

-- nog.wbind = create_bind_fn("w")
-- nog.wbind_tbl = create_bind_tbl_fn("w")

-- nog.components = {}

-- nog.components.datetime = function(format)
--   return {
--     name = "Datetime",
--     render = function()
--       return {{
--         text = nog.fmt_datetime(format),
--       }}
--     end
--   }
-- end

-- nog.components.padding = function(amount)
--   return {
--     name = "Padding",
--     render = function()
--       return {{
--         text = string.rep(" ", amount),
--       }}
--     end
--   }
-- end

-- nog.components.active_mode = function()
--   return {
--     name = "ActiveMode",
--     render = function()
--       local mode
--       if current_mode ~= nil then
--         mode = current_mode .. " is active"
--       end
--       return {{
--         text = mode or "",
--       }}
--     end
--   }
-- end

-- nog.components.current_window = function(max_width)
--   max_width = max_width or 0

--   return {
--     name = "CurrentWindow",
--     render = function(display_id)
--       local win_id = nog.get_focused_win_of_display(display_id)

--       if not win_id then
--         return {{ text = "" }}
--       end
      
--       local title = win_id and nog.get_win_title(win_id) or ""

--       if max_width ~= 0 then
--         title = title:sub(1, max_width)
--       end

--       return {{
--         text = title,
--       }}
--     end
--   }
-- end

-- nog.components.split_direction = function(values)
--   return {
--     name = "SplitDirection",
--     render = function(display_id)
--       local ws_id = nog.get_focused_ws_of_display(display_id)
      
--       if not ws_id then
--         return {{ text = "" }}
--       end

--       local info = nog.get_ws_info(ws_id)

--       return {{
--         text = info.split_direction == "Vertical" and values[1] or values[2],
--       }}
--     end
--   }
-- end

-- nog.components.fullscreen_indicator = function(indicator)
--   return {
--     name = "FullscreenIndicator",
--     render = function(display_id)
--       local ws_id = nog.get_focused_ws_of_display(display_id)

--       if not ws_id then
--         return {{ text = "" }}
--       end

--       local info = nog.get_ws_info(ws_id)

--       return {{
--         text = info.is_fullscreen and indicator or "",
--       }}
--     end
--   }
-- end

-- -- This is used to create a proxy table which notifies nog when a config value changes
-- function create_proxy(path)
--   path = path or {}

--   local prefix = "nog.config"
--   local tbl = nog.config
--   local parts_len = #path
--   local proxy = {}

--   for _, part in ipairs(path) do
--     prefix = prefix .. "." .. part
--     tbl = tbl[part]
--   end

--   setmetatable(proxy, {
--     __index = tbl,
--     __newindex = function(t, k, v)
--       if nog.config.enable_hot_reloading then
--         nog.__on_config_updated(prefix, k, v, nog.__is_setup)
--       end
--       tbl[k] = v
--     end
--   })

--   local tmp_tbl = nog
--   -- name of the field that gets replaced
--   local last_part = "config"

--   for i, part in ipairs(path) do
--     if i == 1 then
--       tmp_tbl = nog.config
--     end
--     if i == parts_len then
--       last_part = part
--       break
--     else
--       tmp_tbl = tmp_tbl[part]
--     end
--   end

--   tmp_tbl[last_part] = proxy
-- end

-- create_proxy({"bar", "components"})
-- create_proxy({"bar"})
-- create_proxy({"rules"})
-- create_proxy({"workspaces"})
-- create_proxy({})

-- nog.config.bar.components = {
--   left = {
--     nog.components.workspaces()
--   },
--   center = {
--     nog.components.datetime("%T")
--   },
--   right = {
--     nog.components.active_mode(),
--     nog.components.padding(5),
--     nog.components.split_direction("V", "H"),
--     nog.components.padding(5),
--     nog.components.datetime("%e %b %Y"),
--     nog.components.padding(1),
--   }
-- }

