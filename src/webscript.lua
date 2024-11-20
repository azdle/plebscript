-- This is a script that builds a https://webscript.io/ -like environment on top
-- of the more raw interface that's directly exposed.
WS = {}

function WS.build_request()

end

function WS.process_response(...)
  local resp = {...}

  local body
  local headers
  local statuscode

  assert(pairs, "pairs was nil")
  assert(table, "table was nil")
  assert(table.remove, "table.remove was nil")

  -- look for body
  for i, val in pairs(resp) do
    if type(val) == "string" then
      body = table.remove(resp, i)
      break
    end
  end

  if body == nil then
    -- look for json body
    for i, val in pairs(resp) do
      if type(val) == "table" then
        body = table.remove(resp, i)
        break
      end
    end
  end

  -- look for headers
  for i, val in pairs(resp) do
    if type(val) == "table" then
      headers = table.remove(resp, i)
      break
    end
  end

  -- look for status code
  for i, val in pairs(resp) do
    if type(val) == "number" then
      status_code = table.remove(resp, i)
      break
    end
  end

  if json_body ~= nil then
    headers = headers or {}
    headers["Content-Type"] = "application/json"
  end

  return {
    body = body,
    headers = headers,
    status_code = status_code,
  }
end

WS.env = {
    -- _G = _G,
    -- form = request.form, -- A table consisting of form data posted to your script. This field is only present if the request has a Content-Type of multipart/form-data or application/x-www-form-urlencode and the body is successfully parsed as form-encoded.
    -- query = request.query, -- A table consisting of query string parameters from the request's URL. For example, the query string ?color=red&number;=3 will result in a query table of {color="red", number="3"}.
    -- querystring = request.querystring, -- The raw query string from the URL. Using the previous example, the querystring value would be "color=red&number;=3".
    -- files = request.files, -- A table consisting of files included in a form post. For each included file, the key is the name of the form's input field, and the value is a table consisting of type (the content-type of the uploaded file), filename (original file name), and content (the raw contents of the file).
    -- body = request.body, -- The content of the request, after it's been decoded as needed (e.g. decompressed as specified by a Content-Encoding header).
    -- method = request.method, -- The HTTP request method. (E.g., GET, POST, ...)
    -- remote_addr = request.remote_addr, -- The request's origin IP address.
    -- scheme = request.scheme, -- The URL scheme. (E.g., http or https.)
    -- port = request.port, -- The URL's port. (E.g., 80 or 443.)
    -- path = request.path, -- The URL's path. (E.g., for the URL http://example.webscript.io/foo/bar, the path is /foo/bar.)
    -- headers = request.headers, -- A table of the HTTP request headers. Keys are "train-cased," like Content-Type.
    request = request,
}

return WS
