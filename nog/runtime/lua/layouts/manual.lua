return function()
  return function(graph, event, win_id)
    if event == "created" then
      graph:add_window_node(nil, win_id)
    elseif event == "deleted" or event == "minimized" then
      graph:del_window_node(win_id)
    end
  end
end
