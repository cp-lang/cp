// Note: Async functions currently require calling builtins directly or 
// compiler support for nested CPS. Direct builtins are used here for clarity.

export fn readFileSync(path: string): string {
    return @fs_read_file(path);
}

export fn writeFileSync(path: string, content: string): void {
    @fs_write_file(path, content);
}

export fn exists(path: string): bool {
    return @fs_exists(path);
}

export fn mkdir(path: string): void {
    @fs_mkdir(path);
}

export fn remove(path: string): void {
    @fs_remove(path);
}

export fn copy(src: string, dest: string): void {
    @fs_copy(src, dest);
}
