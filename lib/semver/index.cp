// lib/semver/index.cp
export { Version } from "./version";
export { SemverRange, Comparator, Op } from "./range";

pub const Semver = struct {
    pub const parse = Version.parse;
    pub const compare = Version.compare;
    pub const satisfies = SemverRange.satisfies;
    pub const satisfiesStr = SemverRange.satisfiesStr;
};

pub const semver = Semver;
