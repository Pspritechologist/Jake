function TAGS.my_tag()
	return "That is awesome"
end

-- print(package.path)

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

-- function TAGS.liquid_context(word)
-- 	data = TEMPLATE.data.title
-- 	TEMPLATE.data.title.owo = 'awa'

-- 	return TEMPLATE.data.title() .. " " .. word
-- end

local rands = {}

for i, file in ipairs(SITE.files) do
	table.insert(rands, math.random(1, 50))
	if rands[table.maxn(rands)] == 1 then
		file:ignore()
		goto continue
	end

	if file.source.ext == "md" then
		file.path.ext = "html"
	end

	for k, v in pairs(file.data) do
		if k == "colors" then
			local color_map = {
				red		= { 255, 0  , 0   },
				green	= { 0  , 255, 0   },
				blue	= { 0  , 0  , 255 },
				yellow	= { 255, 255, 0   },
				orange	= { 255, 165, 0   },
				purple	= { 128, 0  , 128 },
				magenta	= { 255, 0  , 255 },
				cyan	= { 0  , 255, 255 },
			}

			local colors = {}
			for _, color in ipairs(v) do
				local rgb = color_map[color]
				if rgb then
					table.insert(colors, rgb)
				end
			end

			file.data.colors = colors
		end
	end

	::continue::
end

print(table.concat(rands, ", "))

-- local page = File.new()
-- page.content = [[
-- 	This page was generated through Lua :)

-- 	{{ some_value }}
-- 	{{ wow }}
-- ]]
-- page.data.some_value = "This is a value from Lua!"
-- page.data.wow = "Wowza!"

-- page.path = "owo/uwu.html" --[[@as Path]]

-- table.insert(SITE.files, page)

table.insert(SITE.files, File.new {
	content = [[
		This page was generated through Lua :)

		{{ some_value }}
		{{ wow }}
	]],
	data = {
		some_value = "This is a value from Lua!",
		wow = "Wowza!"
	},
	output = "owo/uwu.html"
})
