-- What a script written for the Windows client's Lua 5.1 may use and this client's interpreter does not have
-- of itself, run before the script (client/src/ai.rs). The client's own AI.lua and AI_M.lua need none of it;
-- the players' own scripts (AI\USER_AI) do.
--
-- Names Lua 5.1 has: these are its own definitions of them.
unpack = unpack or table.unpack
loadstring = loadstring or load
table.getn = table.getn or function(list) return #list end
math.mod = math.mod or math.fmod
string.gfind = string.gfind or string.gmatch
print = print or function() end

-- A page has no clock of the system for a script and no folder to keep files in. The time is the tick the
-- client tells (`GetTick`), in seconds; the date is not told.
os = os or {}
os.time = os.time or function() return math.floor(GetTick() / 1000) end
os.clock = os.clock or function() return GetTick() / 1000 end
os.date = os.date or function() return "" end

-- Files: one opened to be read is a file of the client's AI folder as the gateway gave it (`ReadFile`, given
-- by the client); one written is kept for as long as the script runs and can be read back, and is not
-- written anywhere. Enough of Lua's files for scripts that keep notes: read("*a"), read("*l"), lines, seek,
-- write, close.
local kept = {}

local function reader(text)
	local file = { at = 1 }
	function file:read(what)
		if what == "*a" or what == "a" then
			local rest = string.sub(text, self.at)
			self.at = string.len(text) + 1
			return rest
		end
		if self.at > string.len(text) then return nil end
		local stop = string.find(text, "\n", self.at, true)
		local line = string.sub(text, self.at, (stop or string.len(text) + 1) - 1)
		self.at = (stop or string.len(text)) + 1
		return (string.gsub(line, "\r$", ""))
	end
	function file:lines() return function() return self:read("*l") end end
	function file:seek(whence, offset)
		if whence == "set" then self.at = 1 elseif whence == "end" then self.at = string.len(text) + 1 end
		self.at = self.at + (offset or 0)
		return self.at - 1
	end
	function file:write() return nil, "the file is open to be read" end
	function file:close() return true end
	return file
end

local function writer(name, before)
	local file = { parts = { before } }
	function file:write(...)
		for _, part in ipairs({ ... }) do table.insert(self.parts, tostring(part)) end
		kept[name] = table.concat(self.parts)
		return self
	end
	function file:read() return nil end
	function file:lines() return function() return nil end end
	function file:flush() return self end
	function file:close() kept[name] = table.concat(self.parts) return true end
	return file
end

io = io or {}
io.open = function(name, mode)
	local path = string.lower(string.gsub(name, "\\", "/"))
	local how = string.sub(mode or "r", 1, 1)
	if how == "r" then
		local text = kept[path] or ReadFile(name)
		if text == nil then return nil, name .. ": No such file or directory" end
		return reader(text)
	end
	return writer(path, how == "a" and (kept[path] or ReadFile(name)) or "")
end
io.lines = function(name)
	local file = assert(io.open(name, "r"))
	return file:lines()
end
