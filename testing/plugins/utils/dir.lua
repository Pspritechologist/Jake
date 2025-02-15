local ffi = require "ffi"

ffi.cdef[[
    struct dirent {
        unsigned long int d_ino;
        long int d_off;
        unsigned short d_reclen;
        unsigned char  d_type;
        char name[256];
    };
    typedef struct DIR DIR;

	DIR *opendir(const char *name);
	int closedir(DIR *dir);
	void rewinddir(DIR *dir);
	struct dirent *readdir(DIR *dirp);
    int access(const char *path, int amode);
    int mkdir(const char *path, int mode);

	char *mkdtemp(char *template);

	char *strerror(int errnum);
]]

local mod = {}

---@param path string
---@return ffi.cdata*
local function open_dir(path)
	local dir = ffi.C.opendir(path)

	if dir == nil then
		error("error opening directory '" .. path .. "' (" .. ffi.string(ffi.C.strerror(ffi.errno())) .. ")")
	end

	return ffi.gc(dir, ffi.C.closedir)
end

---@param path Path | string
---@return Iterator<string?>?
function mod.iter_dir(path)
	local dir = open_dir("/" .. tostring(path))

	local function next()
		local entry = ffi.C.readdir(dir)

		if entry == nil then
			return nil
		end

		local result = ffi.string(entry.name)
		if result == '.' or result == '..' then
			return next()
		end

		return ffi.string(entry.name)
	end

	return next
end

---@param path Path | string
---@return boolean
function mod.exists(path)
	return ffi.C.access("/" .. tostring(path), 0) == 0
end

---@param path Path | string
---@param mode string?
function mod.mkdir(path, mode)
	if ffi.C.mkdir("/" .. tostring(path), tonumber(mode or "755", 8)) ~= 0 then
		error("error creating directory '" .. path .. "' (" .. ffi.string(ffi.C.strerror(ffi.errno())) .. ")")
	end
end

---@param path Path | string
---@return string
function mod.temp_dir(path)
	path = Path.new(path)
	path.name = path.name .. "XXXXXX"
	path = "/" .. tostring(path)

	local template = ffi.new("char[?]", #path + 1)
	ffi.copy(template, path)

	local result = ffi.C.mkdtemp(template)

	if result == nil then
		error("error creating temporary directory '" .. tostring(path) .. "' (" .. ffi.string(ffi.C.strerror(ffi.errno())) .. ")")
	end

	return ffi.string(result)
end

return mod
