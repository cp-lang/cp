pub const DirInfo = struct {
    id: i32,
    name: []const u8,
};

pub fn get_mock_dir() DirInfo {
    return .{ .id = 88, .name = "mock_dir" };
}
