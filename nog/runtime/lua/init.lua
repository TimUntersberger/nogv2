-- TODO: update package.path
package.cpath = nog.runtime_path .. "/dll/?.dll;" .. package.cpath
package.path = nog.config_path .. "/config/?.lua;" .. package.path

function nog.execute_runtime_file(path)
  return dofile(nog.runtime_path .. "/lua/" .. path)
end

nog.execute_runtime_file "util.lua"

nog.uv = require 'luv'
nog.components = nog.execute_runtime_file "components/init.lua"
nog.inspect = nog.execute_runtime_file "inspect.lua"

nog.layouts = {}
nog.layouts.master_slave = nog.execute_runtime_file("layouts/master_slave.lua")
nog.layouts.manual = nog.execute_runtime_file("layouts/manual.lua")

local ws_to_layout = {}

function nog.__organize(ws_id, layout_name)
  if ws_to_layout[ws_id] == nil then
    ws_to_layout[ws_id] = nog.layouts[layout_name]()
  end

  return ws_to_layout[ws_id]
end

nog.execute_runtime_file "keybindings.lua"
nog.execute_runtime_file "package_loader.lua"

nog.bar_set_layout {
  left = {},
  center = {},
  right = {}
}
