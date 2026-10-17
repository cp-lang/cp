/**
 * CP JSON Serializer (cap:json)
 * Zero-overhead compile-time reflection JSON serialization
 */

export class JSON {
    fn stringify(this, obj: any): string {
        return @json_stringify(obj);
    }
}
export const json = new JSON();
