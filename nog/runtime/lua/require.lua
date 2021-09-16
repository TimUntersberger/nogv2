-- This custom require loader will try to to find the package in the installed plugins.

local plugins = {}

table.insert(package.loaders, function(package_path)
  local plugins_path = nog.config_path .. "\\plugins"
  local dir_handle = nog.uv.fs_scandir(plugins_path)

  if not dir_handle then 
    return "Plugins directory doesn't exist"
  end

  local results = {}

  for _, v in ipairs(plugins) do
    local search_path = plugins_path .. "\\" .. v .. "\\lua\\?.lua"
    local file_path, err = package.searchpath(package_path, search_path)

    if file_path then
      return function()
        return dofile(file_path)
      end
    end

    table.insert(results, err)
  end

  return table.concat(results, "\n")
end)

function nog.plugin_register(name)
  table.insert(plugins, name)
end
