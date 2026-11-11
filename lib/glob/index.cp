// lib/glob/index.cp
import { Matcher } from "./matcher";

pub const Glob = struct {
    pub const match = Matcher.match;
    pub const hasMagic = Matcher.hasMagic;
};

pub const glob = Glob;
