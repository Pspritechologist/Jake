function TAGS.my_tag()
	return "That is awesome"
end

local titlecase = require "testing.plugins.titlecase"

function FILTERS.titlecase(input)
	if not input then
		return ""
	end

	return titlecase(input)
end

function TAGS.test_tag(name, ...)
	local result = {}
	for i, v in ipairs({...}) do
		table.insert(result, name .. " is not a " .. v .. ".")
	end
	table.insert(result, name .. " is " .. name .. "!")

	return table.concat(result, "  \n")
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
