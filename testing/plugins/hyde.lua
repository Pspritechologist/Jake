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
