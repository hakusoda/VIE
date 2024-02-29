--!strict
local Selection = game:GetService("Selection")
local HttpService = game:GetService("HttpService")
local ScriptEditorService = game:GetService("ScriptEditorService")

local root = script.Parent
local plugin: Plugin = plugin
local toolbar = plugin:CreateToolbar("VIE")
local functions = root.functions

local get_full_api_dump = require(functions.get_full_api_dump)

local api_dump = get_full_api_dump()

local import_button = toolbar:CreateButton("vie_import", "", "rbxassetid://13241771148", "Import") :: PluginToolbarButton
import_button.ClickableWhenViewportHidden = true
import_button.Click:Connect(function()
	
end)

type VieInstance = {
	name: string,
	class: string,
	source: string?,
	children: { VieInstance }?,
	properties: { [string]: any }?,
	attributes: { [string]: any }?
}

function get_api_dump_class(class: string): get_full_api_dump.RobloxApiDumpClass?
	for _,item in api_dump.Classes do
		if item.Name == class then
			return item
		end
	end
	
	return nil
end

function serialise_udim(udim: UDim)
	return { udim.Scale, udim.Offset }
end

function serialise_datatype(value: any): any
	local data_type = typeof(value)
	if data_type == "CFrame" or data_type == "BrickColor" or data_type == "ColorSequence" or data_type == "Rect" then
		return tostring(value)
	elseif data_type == "Vector3" then
		return { value.X, value.Y, value.Z }
	elseif data_type == "Vector2" then
		return { value.X, value.Y }
	elseif data_type == "Color3" then
		return { math.floor(value.R * 255), math.floor(value.G * 255), math.floor(value.B * 255) }
	elseif data_type == "Font" then
		return {
			style = value.Style.Name,
			family = value.Family,
			weight = value.Weight.Name
		}
	elseif data_type == "UDim" then
		return serialise_udim(value)
	elseif data_type == "UDim2" then
		return { serialise_udim(value.X), serialise_udim(value.Y) }
	elseif data_type == "EnumItem" then
		-- while the item value would be more efficient, it makes instance files less human(?) readable.
		return (value :: EnumItem).Name
	end
	
	return value
end

local default_properties: { [string]: { [string]: any } } = {}
function get_default_properties(instance: Instance): { [string]: any }
	local class_name = instance.ClassName
	local existing = default_properties[class_name]
	if existing then
		return existing
	end
	
	local class = assert(get_api_dump_class(class_name))
	local instance: Instance = Instance.new(class_name)
	local properties = get_raw_properties(instance, {}, class, true)
	default_properties[class_name] = properties
	
	instance:Destroy()
	instance = nil :: any
	
	return properties
end

function get_raw_properties(instance: Instance, properties: { [string]: any }, class: get_full_api_dump.RobloxApiDumpClass, ignore_defaults: boolean?)
	for _,member in class.Members do
		local member_name = member.Name
		if member_name ~= "Name" and member.MemberType == "Property" and member.Serialization.CanSave then
			local read_security: get_full_api_dump.SecurityLevel = member.Security.Read
			if (read_security == "None" or read_security == "PluginSecurity") and (not member.Tags or (not table.find(member.Tags, "NotScriptable") and not table.find(member.Tags, "ReadOnly"))) then
				local value = (instance :: any)[member_name]
				if ignore_defaults or value ~= get_default_properties(instance)[member_name] then
					properties[member.Name] = value
				end
			end
		end
	end

	local super_class_name = class.Superclass
	if super_class_name ~= "<<<ROOT>>>" then
		local super_class = get_api_dump_class(super_class_name)
		if super_class then
			get_raw_properties(instance, properties, super_class, ignore_defaults)
		end
	end

	return properties
end

function get_appropriate_kind(type_of: string, api_type: string): string
	if type_of == "number" then
		return api_type
	end
	return type_of
end

function get_class_property(class: get_full_api_dump.RobloxApiDumpClass, property_name: string): get_full_api_dump.RobloxApiDumpClassMemberProperty
	for _,member in class.Members do
		if member.MemberType == "Property" and member.Name == property_name then
			return member
		end
	end
	
	local super_class_name = class.Superclass
	if super_class_name ~= "<<<ROOT>>>" then
		local super_class = get_api_dump_class(super_class_name)
		if super_class then
			return get_class_property(super_class, property_name)
		end
	end
	
	error("did not find")
end

function get_properties(instance: Instance, class: get_full_api_dump.RobloxApiDumpClass, ignore_defaults: boolean?)
	local properties = get_raw_properties(instance, {}, class)
	for key, value in properties do
		properties[key] = {
			kind = get_appropriate_kind(typeof(value), get_class_property(class, key).ValueType.Name),
			value = serialise_datatype(value)
		}
	end
	
	return properties
end

function get_attributes(instance: Instance)
	local attributes = {}
	for key, value in instance:GetAttributes() do
		attributes[key] = {
			kind = typeof(value),
			value = serialise_datatype(value)
		}
	end
	
	return attributes
end

function serialise_instance(instance: Instance): VieInstance
	local name = instance.Name
	assert(string.find(name, "+") ~= 1, "Instance names cannot start with a plus symbol (+), as they are reserved for VIE files.")
	
	local class = instance.ClassName
	local vie: VieInstance = {
		name = name,
		class = class
	}
	local children = {}
	for _,child in instance:GetChildren() do
		if not child:IsA("PackageLink") then
			table.insert(children, serialise_instance(child))
		end
	end
	
	if #children > 0 then
		vie.children = children
	end
	
	if instance:IsA("LuaSourceContainer") then
		vie.source = ScriptEditorService:GetEditorSource(instance)
	end
	
	local api_class = assert(get_api_dump_class(class))
	local properties = get_properties(instance, api_class)
	if next(properties) then
		vie.properties = properties
	end
	
	local attributes = get_attributes(instance)
	if next(attributes) then
		vie.attributes = attributes
	end
	
	return vie
end

local export_button = toolbar:CreateButton("vie_export", "", "rbxassetid://14813993705", "Export") :: PluginToolbarButton
export_button.ClickableWhenViewportHidden = true
export_button.Click:Connect(function()
	local instance = assert(Selection:Get()[1])
	local json = HttpService:JSONEncode({
		kind = "export",
		data = serialise_instance(instance)
	})
	HttpService:RequestAsync({
		Url = "http://localhost:3143",
		Body = json,
		Method = "POST"
	})
end)