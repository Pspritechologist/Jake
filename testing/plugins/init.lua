function TAGS.my_tag()
	return "That is awesome"
end

print(package.path)

local titlecase = require "titlecase"

---@param input any
function FILTERS.titlecase(input)
	if type(input) ~= "string" then
		return input
	end

	return titlecase(input)
end

function TAGS.test_tag(name, ...)
	local result = {}
	for i, v in ipairs({...}) do
		table.insert(result, name .. " is not a " .. v .. ".")
	end
	table.insert(result, name .. " is " .. name .. "!")

	return table.concat(result, "\n\n")
end

-- A list containing 15 different fruits
local fruits = {
	"apple", "banana", "cherry", "date", "elderberry", "fig", "grape", "honeydew", "imbe", "jackfruit", "kiwi", "lemon", "mango", "nectarine", "orange"
}

-- Iterate over the list and make a tag for each one.
for _, fruit in ipairs(fruits) do
	TAGS["fruit_" .. fruit] = function()
		return fruit .. "s are good for you!"
	end
end

function TAGS.liquid_context(word)
	data = TEMPLATE.data.title
	TEMPLATE.data.title.owo = 'awa'

	return TEMPLATE.data.title() .. " " .. word
end

-- proj = SITE.project_dir
-- print(proj["end"])
-- print(proj.ext)
-- print(proj.name)
-- print(proj.parent)
-- print(proj.parent.parent)
-- print(proj.is_file)
-- print(proj.exists)
-- print(proj.is_absolute)
-- print(proj:join(SITE.layout_dir, "owo"))

for i, file in ipairs(SITE.files) do
	print(i)
	if file.source.ext == "md" then
		file.path.ext = "html"
	end

	print("", file.source, " -> ", file.path)
	for k, v in pairs(file.data) do
		print("", "", k, v)
	end
end
