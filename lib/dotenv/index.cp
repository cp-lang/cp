// lib/dotenv/index.cp
import { DotEnv, Entry } from "./loader";

pub const dotenv = struct {
    pub const parse = DotEnv.parse;
};
