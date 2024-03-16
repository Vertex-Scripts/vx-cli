fx_version "cerulean"

description "Advanced cardealer script."
author "Vertex Scripts"
version "0.0.0"

lua54 "yes"

ui_page "web/dist/index.html"

games {
	"gta5"
}

dependencies {
	"vx_lib"
}

server_scripts {
	"@oxmysql/lib/MySQL.lua",
	"customize/server.lua",
	"server/**/*"
}

client_scripts {
	"client/nui.lua",
	"client/customization.lua",
	"client/test.lua",
	"client/client.lua",
}

shared_scripts {
	"@vx_lib/init.lua",
	"strings.lua",
	"config.lua"
}

files {
	"web/dist/index.html",
	"web/dist/**/*"
}

escrow_ignore {
	"web/**/*",
	"customize/**/*.lua",
	"strings.lua",
	"config.lua",
	"types.lua"
}

vx_ignore {
    "src/server/**"
}