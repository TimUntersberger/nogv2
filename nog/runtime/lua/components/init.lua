return {
  workspaces = nog.execute_runtime_file("components/workspaces.lua"),
  current_window = nog.execute_runtime_file("components/current_window.lua"),
  padding = nog.execute_runtime_file("components/padding.lua"),
}
