function dump_table(t, indent, acc)
  indent = indent and (indent + 2) or 0
  acc = acc or {}

  for k,v in pairs(t) do
    if type(v) == table then
      dump_table(v, indent, acc)
    else
      table.insert(k, v)
    end
  
  end
end

-- return {
--     form = form,
--     query = query,
--     querystring = querystring,
--     files = files,
--     body = body,
--     method = method,
--     remote_addr = remote_addr,
--     scheme = scheme,
--     port = port,
--     path = path,
--     headers = headers,
-- }

return request
