local file = request.files.thefile
return string.format(
 'You uploaded a file with type "%s" called "%s" containing %d bytes.',
 file.type, file.filename, #file.content)
 
 [[
 <form action="https://examples.webscript.io/upload" method="post"
 enctype="multipart/form-data">
 <label for="thefile">Choose a file to upload:</label>
 <input name="thefile" type="file"/>
 <button type="submit" class="btn btn-primary">Upload</button>
</form>
]]
