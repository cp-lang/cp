enum FileErr {
    FileNotFound,
    PermDenied,
    ReadEOF
}

fn readFile(path: string) : ?string ! FileErr {
    if (path == "") return Err(FileErr.FileNotFound);
    return path;
}

fn main() {
    let res = readFile("data.txt");
    // Error: Non-exhaustive switch (missing Err(FileErr.ReadEOF) and null branch, no `*` wildcard)
    let out = switch res {
        content => content,
        Err(FileErr.FileNotFound) => "not_found",
        Err(FileErr.PermDenied) => "perm_denied"
    };
    @print("{s}", out);
}
