local function titlecase(input)
    local small_words = {
        a = true, an = true, ['and'] = true, as = true, at = true, but = true, by = true,
        en = true, ['for'] = true, ['if'] = true, ['in'] = true, of = true, on = true, ['or'] = true,
        the = true, to = true, v = true, ['v.'] = true, via = true, vs = true, ['vs.'] = true
    }

    local function smart_capitalize(word)
        local leading, first_char, rest = word:match("^(['\"%(%[]*)([a-zA-Z])(.+)$")
        if first_char then
            if not rest:match("[A-Z]" or "%..") then
                first_char = first_char:upper()
            end
            return (leading or "") .. first_char .. rest
        end
        return word
    end

    local words = {}
    for word in input:gsub("_", " "):gmatch("%S+") do
        local stripped_word = word:match("%w*")
        if small_words[stripped_word:lower()] then
            table.insert(words, word:lower())
        else
            table.insert(words, smart_capitalize(word))
        end
    end

    if #words > 0 then
        words[1] = smart_capitalize(words[1])
        words[#words] = smart_capitalize(words[#words])
    end

    local result = table.concat(words, " ")
    result = result:gsub(":%s?([^%w]*[%a%d]+[^%w]*)%s", function(match)
        return ": " .. smart_capitalize(match)
    end)

    return result
end

return titlecase
