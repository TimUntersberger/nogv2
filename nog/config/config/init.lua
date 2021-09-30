nog.nbind("alt+ctrl+f", function()
  if nog.win_is_managed(nil) then
    nog.win_unmanage(nil)
  else
    nog.win_manage(nil)
  end
end)

nog.gbind("alt+escape", function()
  if nog.is_awake() then
    nog.hibernate()
  else
    nog.awake()
  end
end)

nog.nbind("alt+ctrl+r", function()
  dofile(nog.config_path .. "\\lua\\config.lua")
end)

nog.nbind("alt+space", function()
  nog.open_menu()
end)

nog.nbind("alt+F1", function()
  nog.update_window_layout()
end)

nog.nbind_tbl("alt", nog.change_ws, nog.range(9))
nog.nbind("alt+0", function()
  nog.change_ws(10)
end)

nog.nbind("alt+x", function()
  nog.exit()
end)

nog.nbind("alt+h", function()
  nog.ws_focus(nil, "left")
end)

nog.nbind("alt+j", function()
  nog.ws_focus(nil, "down")
end)

nog.nbind("alt+l", function()
  nog.ws_focus(nil, "right")
end)

nog.nbind("alt+k", function()
  nog.ws_focus(nil, "up")
end)

nog.nbind("alt+h", function()
  nog.ws_focus(nil, "left")
end)

nog.nbind("alt+j", function()
  nog.ws_focus(nil, "down")
end)

nog.nbind("alt+l", function()
  nog.ws_focus(nil, "right")
end)

nog.nbind("alt+k", function()
  nog.ws_focus(nil, "up")
end)

nog.nbind("ctrl+alt+h", function()
  nog.ws_swap(nil, "left")
end)

nog.nbind("ctrl+alt+j", function()
  nog.ws_swap(nil, "down")
end)

nog.nbind("ctrl+alt+l", function()
  nog.ws_swap(nil, "right")
end)

nog.nbind("ctrl+alt+k", function()
  nog.ws_swap(nil, "up")
end)

nog.nbind("alt+q", function()
  nog.win_close(nil)
end)

nog.config.color = 0x2e3440
nog.config.font_name = "CaskaydiaCove NF"
nog.config.font_size = 18
nog.config.bar_height = 20

nog.bar_set_layout {
  left = {
    nog.components.workspaces(),
    nog.components.padding(1),
    nog.components.current_window()
  },
  center = {
    nog.components.datetime("%H:%M:%S")
  },
  right = {
    nog.components.datetime("%d.%m.%Y"),
    nog.components.padding(1),
  }
}
