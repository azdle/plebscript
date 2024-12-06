if request.method ~= "GET" then
  return 405, "Method Not Allowed: " .. request.method
end

return {
  args = request.query,
  headers = request.headers,
  origin = request.remote_addr,
  url = request.scheme .. "://" .. request.headers.host .. request.path,
}
