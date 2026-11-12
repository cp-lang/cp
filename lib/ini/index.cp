// lib/ini/index.cp
export { Ini, Property } from "./parser";

pub const ini = struct {
    pub const parse = Ini.parse;
};
