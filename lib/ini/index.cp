// lib/ini/index.cp
import { Ini, Property } from "./parser";

pub const ini = struct {
    pub const parse = Ini.parse;
};
