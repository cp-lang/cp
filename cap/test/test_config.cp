// cap/test/test_config.cp
const std = @import("std");
import { test, expect_eq } from "cap:test";
import { dotenv, DotEnv } from "cap:dotenv";
import { ini, Ini } from "cap:ini";

fn testDotEnvParsing(): !void {
    const env_content = "# App Configuration\nAPP_NAME=Antigravity\nPORT=8080\nDEBUG=true\n\n# Export syntax\nexport SECRET_KEY=\"top_secret_key\"\n\n# Escaped values\nGREETING=\"Hello\\nWorld\"\n\n# Single quoted\nRAW_PATH='C:\\Users\\App\\data'\n\n# Inline comment\nTIMEOUT=30 # seconds\n";

    const env = DotEnv.parse(env_content);

    expect_eq(std.mem.eql(u8, env.get("APP_NAME").?, "Antigravity"), true);
    expect_eq(std.mem.eql(u8, env.get("PORT").?, "8080"), true);
    expect_eq(std.mem.eql(u8, env.get("DEBUG").?, "true"), true);
    expect_eq(std.mem.eql(u8, env.get("SECRET_KEY").?, "top_secret_key"), true);
    expect_eq(std.mem.eql(u8, env.get("GREETING").?, "Hello\nWorld"), true);
    expect_eq(std.mem.eql(u8, env.get("RAW_PATH").?, "C:\\Users\\App\\data"), true);
    expect_eq(std.mem.eql(u8, env.get("TIMEOUT").?, "30"), true);

    expect_eq(env.has("APP_NAME"), true);
    expect_eq(env.has("NON_EXISTENT"), false);
    expect_eq(std.mem.eql(u8, env.getOr("MISSING_VAR", "default_val"), "default_val"), true);
}

fn testIniParsing(): !void {
    const ini_content = "; Global section settings\ntitle = My Application\nversion = 1.0.0\n\n[server]\nhost = 127.0.0.1\nport = 3000\nenable_ssl = false\n\n[database]\nname = \"production_db\"\nmax_connections = 100 ; max pool connections\n";

    const conf = Ini.parse(ini_content);

    // Global properties
    expect_eq(std.mem.eql(u8, conf.getGlobal("title").?, "My Application"), true);
    expect_eq(std.mem.eql(u8, conf.getGlobal("version").?, "1.0.0"), true);

    // Section properties
    expect_eq(std.mem.eql(u8, conf.get("server", "host").?, "127.0.0.1"), true);
    expect_eq(std.mem.eql(u8, conf.get("server", "port").?, "3000"), true);
    expect_eq(std.mem.eql(u8, conf.get("server", "enable_ssl").?, "false"), true);

    // Database section
    expect_eq(std.mem.eql(u8, conf.get("database", "name").?, "production_db"), true);
    expect_eq(std.mem.eql(u8, conf.get("database", "max_connections").?, "100"), true);

    // Section presence and defaults
    expect_eq(conf.hasSection("server"), true);
    expect_eq(conf.hasSection("redis"), false);
    expect_eq(std.mem.eql(u8, conf.getOr("server", "workers", "4"), "4"), true);
}

fn main(): void {
    test("DotEnv configuration parsing and variable retrieval", () => {
        try testDotEnvParsing();
    });
    test("INI configuration parsing and section management", () => {
        try testIniParsing();
    });
}
