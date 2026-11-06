export fn randomUUID(): string {
    return @crypto_uuid();
}

export fn hash(data: string): string {
    return @crypto_hash(data);
}
