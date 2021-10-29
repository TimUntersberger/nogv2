return function()
  return function ()
    local dsp = nog.dsp_get_focused()
    local ws = nog.dsp_get_focused_ws(dsp)

    if not ws then
      return ""
    end

    local win = nog.ws_get_focused_win(ws)

    if win then
      return nog.win_get_title(win)
    end

    return ""
  end
end
