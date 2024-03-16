setmetatable(_G, {
	__index = function(t, k)
		local raw = rawget(t, k)
		if raw then
			return raw
		end

		return function(value)
			local newK = k
			if type(value) == 'table' then
				if k:sub(-1) == 's' then
					newK = k:sub(1, -2)
				end

				for _, v in ipairs(value) do
				    _INTERCEPTOR(newK, v)
				end
			else
				_INTERCEPTOR(k, value)
			end

			return function()end
		end
	end
})
