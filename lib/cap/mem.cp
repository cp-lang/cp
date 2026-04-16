export class Arena {
    fn alloc(this, size: usize): *void {
        return @alloc(size);
    }
    fn free(this): void {
        @free();
    }
}

export fn get_arena(): Arena {
    return @get_arena();
}

export class SharedTensor {
    size: usize,
    fn init(size: usize): SharedTensor {
        return @shared_tensor_init([size]);
    }
}