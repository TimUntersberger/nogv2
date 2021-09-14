return function()
  return function ()
    return nog.tbl_map(
      nog.tbl_filter(
        nog.ws_get_all(), 
        function(ws_id)
          return nog.dsp_contains_ws(nil, ws_id)
        end
      ), 
      function(ws_id)
        return string.format(" %d ", ws_id)
      end
    )
  end
end
