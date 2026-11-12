/**
 * CP HTTP Request (cap:http)
 */
export class Request {
    pub method: string;
    pub url: string;
    pub body: string;

    pub fn init(this, method: string, url: string, body: string): void {
        this.method = method;
        this.url = url;
        this.body = body;
    }

    pub fn text(this): string {
        return this.body;
    }

    pub fn json(this): any {
        return @json_parse(this.body);
    }
}
