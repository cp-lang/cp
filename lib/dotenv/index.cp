// lib/dotenv/index.cp
export { DotEnv, Entry } from "./loader";

pub const dotenv = struct {
    pub const parse = DotEnv.parse;
};
