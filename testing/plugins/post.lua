local pl_dir = require "pl.dir"

GEN_DIR = Path.join(SITE.project_dir, "generated")

return function()
	for i, entry in ipairs(pl_dir.getallfiles("/" .. tostring(GEN_DIR))) do
		-- os.execute
	end
end
