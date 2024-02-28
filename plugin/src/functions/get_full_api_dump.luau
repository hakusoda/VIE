--!strict
local HttpService = game:GetService("HttpService")
export type RobloxApiDump = {
	Enums: { unknown },
	Version: number, -- always 1, supposedly.
	Classes: { RobloxApiDumpClass }
}

export type RobloxApiDumpClass = {
	Name: string,
	Tags: { string },
	Members: { RobloxApiDumpClassMember },
	Superclass: string,
	MemoryCategory: "Internal" | "Instances" -- unfinished
}
export type RobloxApiDumpClassMember = RobloxApiDumpClassMemberProperty | RobloxApiDumpClassMemberMethod
export type RobloxApiDumpClassMemberBase = {
	Name: string,
	Tags: { string },
	Serialization: {
		CanSave: boolean,
		CanLoad: boolean
	},
	ThreadSafety: ThreadSafety
}
export type RobloxApiDumpClassMemberProperty = RobloxApiDumpClassMemberBase & {
	Default: string,
	Category: string,
	Security: Security,
	ValueType: DataType,
	MemberType: "Property",
	Serialization: {
		CanLoad: boolean,
		CanSave: boolean
	}
}
export type RobloxApiDumpClassMemberMethod = RobloxApiDumpClassMemberBase & {
	Category: string,
	MemberType: "Method",
	Parameters: { unknown },
	ReturnType: DataType
}

-- unfinished
export type DataType = {
	Name: string,
	Category: "Class"
} | {
	Name:
		"int" | "float" |
	"bool" |
	"string", -- unfinished
	Category: "Primitive"
} | {
	Name: "Group" | "Dictionary",
	Category: "Tuple"
}

export type ThreadSafety = "Unknown" | "Unsafe" | "Safe" | "ReadSafe"

export type Security = {
	Read: SecurityLevel,
	Write: SecurityLevel
}
export type SecurityLevel = "None" | "RobloxSecurity" | "PluginSecurity" | "LocalUserSecurity" | "NotAccessibleSecurity"

return function(): RobloxApiDump
	local response = HttpService:GetAsync("https://raw.githubusercontent.com/MaximumADHD/Roblox-Client-Tracker/roblox/Full-API-Dump.json")
	return HttpService:JSONDecode(response)
end