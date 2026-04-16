fn main(): ?void {
    !{
        const MAX = 1024;
        if (MAX < 1000) {
            @compileError("too small");
        }
    }
    @print("Comptime check passed.");
}