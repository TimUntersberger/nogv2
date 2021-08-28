local state = {
  master = nil,
  slave_group = nil,
  slaves = {}
}

function state.remove_slave(self, id)
  self.slaves = nog.tbl_filter(
    self.slaves, 
    function(slave)
      return slave ~= id
    end
  )
end

local function layout(graph, event, win_id)
  if event == "created" then
    local slave_count = #state.slaves
    if state.master == nil then
      state.master = graph:add_window_node(nil, win_id)
    elseif slave_count == 0 then
      state.slave_group = graph:add_column_node(nil)
      table.insert(
        state.slaves, 
        graph:add_window_node(state.slave_group, win_id)
      )
    else
      table.insert(
        state.slaves, 
        graph:add_window_node(state.slave_group, win_id)
      )
    end
  elseif event == "deleted" or event == "minimized" then
    local deleted_id = graph:del_window_node(win_id)

    if state.master == deleted_id then
      state.master = state.slaves[1]
      if state.master then
        graph:move_node(nil, state.master, 0)
        state:remove_slave(state.master)
      end
    else
      state:remove_slave(deleted_id)
    end

    if #state.slaves == 0 and state.slave_group then
      graph:del_node(state.slave_group)
      state.slave_group = nil
    end
  end

  -- print(nog.inspect(state))
end

return layout
