local graph = {}
-- test implementation for master slave layout
local function master_slave_layout(event, grid)
  if event.kind == "MANAGE" then
    local node_count = #graph
    if node_count == 0 then
      local idx = grid:add_node(nil, event.window)
      graph[1] = {
        id = idx,
        win = event.window
      }
    elseif node_count == 1 then
      local idx = grid:add_column_node(nil)
      graph[2] = {
        id = idx,
        children = {}
      }
      idx = grid:add_node(graph[2].id, event.window)
      table.insert(graph[2].children, {
        id = idx,
        win = event.window
      })
    else
      local idx = grid:add_node(graph[2].id, event.window)
      table.insert(graph[2].children, {
        id = idx,
        win = event.window
      })
    end
  elseif event.kind == "UNMANAGE" then
  end
end

local function manual_layout(event, grid)
  if event.kind == "MANAGE" then
    local idx = grid:add_window_node(nil, event.window)
    table.insert(graph[2].children, {
      id = idx,
      win = event.window
    })
  elseif event.kind == "UNMANAGE" then
  end
end

nog.layout = function(graph, event, win_id)
  print(event, win_id)
end
-- inspect(graph)
-- layout({ kind = "MANAGE", window = 1 }, grid)
-- inspect(graph)
-- layout({ kind = "MANAGE", window = 2 }, grid)
-- inspect(graph)
-- layout({ kind = "MANAGE", window = 3 }, grid)
-- inspect(graph)
