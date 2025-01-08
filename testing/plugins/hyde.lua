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
---@type table<string, fun(target: any, named_args: Args, pos_args...: any): any>
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

--- A file object.
---@class File
---@field source Path? The path to the source file, if it exists.
---@field path Path The path this file's output will be generated to.
---@field data table<string, any> The frontmatter data of the file.
--- The content of the file.
--- 
--- If this value is nil, the file cannot be parsed as text.  
--- This typically means the file is binary, such as an image.
---@field content string?
--- True if this file should be written to the output directory.  
--- Setting this to false will prevent the file from being written.
---@field to_write boolean
--- A function to be used in post-processing of the file.  
--- This will be called *after* the file has been
--- rendered, but *before* it is placed in any layouts.
---@field post_proc fun(file: File)
---@field is_text boolean `true` if this file is textual.
---@field is_binary boolean `true` if this file is binary.
File = {}

---@alias FileData { content: string?, data: Args?, output: Path | string?, post_processor: fun(file: File)? }

--- Create a new File object.  
--- This can be used to generate files programmatically.
---@return File
---@param data FileData?
function File.new(data) end

--- Set whether this file should be ignored by Hyde.  
--- If no argument is given, the file is ignored.
---@param ignore boolean?
function File:ignore(ignore) end

--- A path object.
--- 
--- This type implements `tostring`, equality, and indexing.
--- - Indexing can be done by number to get the parts of the path.
---   `path[1]` will return the first part of the path, `path[2]` the second, etc.
--- - `path == path` will only consider normalized elements, eg. `'/foo/bar == /foo/bar/baz/..` is true.
---@class Path
--- The file extension, without leading dot.  
--- Setting this will change the extension of the path.
---@field ext string?
---@field parent Path? The parent directory of this path.
--- The last part of the path, be it file or directory, including extension.  
--- Setting this will change the last part of the path.
---@field last string?
--- The name of the file or directory, without extension.  
--- Setting this will change the name of the file or directory,
--- leaving the extension intact if no new one is supplied.
---@field name string?
---@field [integer] string
---@operator concat: Path | string
---@operator add: Path | string
---@operator len: integer
Path = {}

--- Append a path to this one.  
--- If the path is absolute this will fully replace the current path.
---@param path Path | string?
function Path:push(path) end

--- Create a path that is two or more paths joined together.
---@param ... Path | string?
function Path.join(...) end

--- Create a new Path object.
---@return Path
---@param path Path | string?
function Path.new(path) end
