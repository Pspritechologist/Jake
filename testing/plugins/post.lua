local dir = require("utils").dir

GEN_DIR = Path.join(SITE.project_dir, "generated")
local tsc_dir = Path.join(GEN_DIR, "assets/ts/")
local out_dir = Path.join(SITE.output_dir, "assets/ts/")

return function()
	if not dir.exists(tsc_dir) then return end
	if not dir.exists(out_dir) or true then
		-- dir.mkdir(out_dir)
		dbg(io.open("/" .. tostring(Path.join(tsc_dir, "tsconfig.json")), "w")):write("{ }")
	end

	os.execute(tostring("tsc --p '" .. tsc_dir .. "' --lib esnext,dom --outDir '" .. out_dir .. "'"))
end
