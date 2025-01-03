---@meta Hyde

--- List of tags to be registered with Hyde.
--- 
--- Functions should be added to this table under the
--- desired name of the tag.
---@type table<string, fun(...): any>
TAGS = {}

---@alias Args table<string, any>

--- List of filters to be registered with Hyde.
--- 
--- Functions should be added to this table under the
--- desired name of the filter.
---@type table<string, fun(any, Args, ...): any>
FILTERS = {}

--- Global data for the Hyde project.  
--- Includes config data, all files, paths, etc.
---@class SITE
--- The root directory of the project.
--- Most paths are relative to this.
---@field project_dir Path
--- The source directory of the project.  
--- This is where all the source files are located and what the output will mirror.
---@field source_dir Path
--- The output directory of the project.  
--- This is where the output files will be written to.
---@field output_dir Path
--- The directory where the plugins are located.  
--- Lua plugins are loaded from this directory- you're likely there right now!
---@field plugins_dir Path
--- The directory where the layouts are located.
---@field layout_dir Path
--- The list of all source files in the project.
---@field files File[]
SITE = {}

--- Create a new file object.
---@return File
function SITE.new_file() end

--- A file object.
---@class File
---@field source Path? The path to the source file, if it exists.
---@field path Path The path this file's output will be generated to.
---@field data table<string, any> The frontmatter data of the file.
---@field content string The content of the file.
--- True if this file should be written to the output directory.  
--- Setting this to false will prevent the file from being written.
---@field write boolean
File = {}

--- Set whether this file should be ignored by Hyde.  
--- If no argument is given, the file is ignored.
---@param ignore boolean?
function File:ignore(ignore) end

--- A path object.
---@class Path
--- The file extension, without leading dot.  
--- Setting this will change the extension of the path.
---@field ext string?
---@field parent Path? The parent directory of this path.
--- The last part of the path, be it file or directory, including extension.  
--- Setting this will change the last part of the path.
---@field end string?
--- The name of the file or directory, without extension.  
--- Setting this will change the name of the file or directory, leaving the extension intact.
---@field name string?
---@field is_dir boolean True if this is a directory.
---@field is_file boolean True if this is a file.
---@field is_absolute boolean True if this is an absolute path.
---@field is_relative boolean True if this is a relative path.
---@field exists boolean True if this path exists on the filesystem.
Path = {}

--- Append a path to this one.  
--- If the path is absolute this will fully replace the current path.
---@param path Path | string?
function Path:push(path) end

--- Create a path that is two or more paths joined together.
---@param ... Path | string?
function Path.join(...) end
