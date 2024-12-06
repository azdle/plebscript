if request.method ~= "PATCH" then
  return 405, "Method Not Allowed: " .. request.method
end

return {
  args = request.query,
  data = request.body,
  files = request.files,
  form = request.form,
  headers = request.headers,
  -- TODO: not a thing
  json = request.json,
  origin = request.remote_addr,
  url = request.scheme .. "://" .. request.headers.host .. request.path,
}

