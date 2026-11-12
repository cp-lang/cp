/**
 * CP HTTP Response (cap:http)
 */
export class Response {
    pub status: i32;
    pub body: string;
    pub contentType: string;

    pub fn init(this, body: string): void {
        this.status = 200;
        this.body = body;
        this.contentType = "text/plain";
    }

    pub fn text(this): string {
        return this.body;
    }

    pub fn json(this): any {
        return @json_parse(this.body);
    }

    pub fn ok(this): bool {
        if (this.status >= 200) {
            if (this.status < 300) {
                return true;
            }
        }
        return false;
    }
}
