function I(...)
  print(nog.inspect(...))
end

function keybindings(tbl)
  for mode, bindings in pairs(tbl) do
    for key, cb in pairs(bindings) do
      nog.bind(mode, key, cb)
    end
  end
end

function event_handlers(tbl)
  for event_name, event_handlers in pairs(tbl) do
    nog.on(event_name, function(ev)
      for _, handler in ipairs(event_handlers) do
        if handler.when(ev) then
          handler.action(ev)
          break
        end
      end
    end)
  end
end

nog.config.color = 0x2e3440
nog.config.remove_task_bar = false
nog.config.remove_decorations = false
nog.config.inner_gap = 5
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

keybindings {
  g = {
    ["alt+escape"] = function()
      if nog.is_awake() then
        nog.hibernate()
      else
        nog.awake()
      end
    end
  },
  n = {
    ["alt+m"] = function()
      nog.win_minimize(nil)
    end,
    ["alt+t"] = function()
      if nog.win_is_managed(nil) then
        nog.win_unmanage(nil)
      else
        nog.win_manage(nil)
      end
    end,
    ["alt+ctrl+f"] = function()
      nog.ws_set_fullscreen(1, not nog.ws_is_fullscreen(1))
    end,
    ["alt+ctrl+r"] = function()
      dofile(nog.config_path .. "\\lua\\config.lua")
    end,
    ["alt+space"] = function()
      nog.open_menu()
    end,
    ["alt+x"] = function()
      nog.exit()
    end,
    ["alt+q"] = function()
      nog.win_close(nil)
    end,
    ["alt+h"] = function()
      nog.ws_focus(nil, "left")
    end,
    ["alt+j"] = function()
      nog.ws_focus(nil, "down")
    end,
    ["alt+l"] = function()
      nog.ws_focus(nil, "right")
    end,
    ["alt+k"] = function()
      nog.ws_focus(nil, "up")
    end,
    ["ctrl+alt+h"] = function()
      nog.ws_swap(nil, "left")
    end,
    ["ctrl+alt+j"] = function()
      nog.ws_swap(nil, "down")
    end,
    ["ctrl+alt+l"] = function()
      nog.ws_swap(nil, "right")
    end,
    ["ctrl+alt+k"] = function()
      nog.ws_swap(nil, "up")
    end,
    ["alt+1"] = function()
      nog.change_ws(1)
    end,
    ["alt+2"] = function()
      nog.change_ws(2)
    end,
    ["alt+3"] = function()
      nog.change_ws(3)
    end,
    ["alt+4"] = function()
      nog.change_ws(4)
    end,
    ["alt+5"] = function()
      nog.change_ws(5)
    end,
    ["alt+6"] = function()
      nog.change_ws(6)
    end,
    ["alt+7"] = function()
      nog.change_ws(7)
    end,
    ["alt+8"] = function()
      nog.change_ws(8)
    end,
    ["alt+9"] = function()
      nog.change_ws(9)
    end,
    ["alt+0"] = function()
      nog.change_ws(10)
    end,
    ["alt+ctrl+1"] = function()
      nog.move_win_to_ws(nil, 1)
    end,
    ["alt+ctrl+2"] = function()
      nog.move_win_to_ws(nil, 2)
    end,
    ["alt+ctrl+3"] = function()
      nog.move_win_to_ws(nil, 3)
    end,
    ["alt+ctrl+4"] = function()
      nog.move_win_to_ws(nil, 4)
    end,
    ["alt+ctrl+5"] = function()
      nog.move_win_to_ws(nil, 5)
    end,
    ["alt+ctrl+6"] = function()
      nog.move_win_to_ws(nil, 6)
    end,
    ["alt+ctrl+7"] = function()
      nog.move_win_to_ws(nil, 7)
    end,
    ["alt+ctrl+8"] = function()
      nog.move_win_to_ws(nil, 8)
    end,
    ["alt+ctrl+9"] = function()
      nog.move_win_to_ws(nil, 9)
    end,
    ["alt+ctrl+0"] = function()
      nog.move_win_to_ws(nil, 10)
    end
  }
}

event_handlers {
  manage = {
    {
      when = function(ev)
        local size = nog.win_get_size(ev.win_id)
        return size.width <= 100 or size.height <= 100
      end,
      action = function(ev)
        nog.win_unmanage(ev.win_id)
      end
    },
    {
      when = function(ev)
        local title = nog.win_get_title(ev.win_id)
        return title:find("^.* - Notepad$")
      end,
      action = function(ev)
        if ev.ws_id ~= 2 then
          nog.move_win_to_ws(ev.win_id, 2)
        end
      end
    },
  }
}
