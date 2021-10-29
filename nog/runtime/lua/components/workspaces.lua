return function()
  return function ()
    local dsp = nog.dsp_get_focused()

    return nog.tbl_map(
      nog.tbl_filter(
        nog.ws_get_all(),
        function(ws_id)
          return nog.dsp_contains_ws(nil, ws_id)
        end
      ), 
      function(ws_id)
        -- TODO: support light theme
        local bg = nog.scale_color(nog.config.color, 1.5)

        if nog.dsp_get_focused_ws(dsp) == ws_id then
          bg = nog.scale_color(nog.config.color, 2.0)
        end

        return {
          string.format(" %s ", nog.ws_get_name(ws_id)),
          bg = bg
        }
      end
    )
  end
end
