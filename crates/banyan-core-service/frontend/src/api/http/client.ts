
/**
 * HttpClient is a custom wrapper around fetch api.
 * Exposes get, post and delete methods for JSON.
 */
export class HttpClient {
    /**
   * Performs POST http request with JSON body.
   * @param path
   * @param body serialized JSON
   * @param headers holds request header
   */
    public async post(
        path: string,
        body?: string,
        headers?: HeadersInit,
    ): Promise<Response> {
        return await this.do('POST', path, body, headers);
    }

    /**
   * Performs PATCH http request with JSON body.
   * @param path
   * @param body serialized JSON
   * @param headers holds request header
   */
    public async patch(
        path: string,
        body?: string,
        headers?: HeadersInit,
    ): Promise<Response> {
        return await this.do('PATCH', path, body, headers);
    }

    /**
   * Performs PUT http request with JSON body.
   * @param path
   * @param body serialized JSON
   * @param headers holds request header
    */
    public async put(
        path: string,
        body?: string,
        headers?: HeadersInit,
    ): Promise<Response> {
        return await this.do('PUT', path, body, headers);
    }

    /**
     * Performs GET http request.
     * @param path
     * @param body serialized JSON
     * @param headers holds request header
    */
    public async get(
        path: string,
        body?: string,
        headers?: HeadersInit,
    ): Promise<Response> {
        return await this.do('GET', path, body, headers);
    }

    /**
     * Performs DELETE http request.
     * @param path
     * @param body serialized JSON    headers?: HeadersInit,

    */
    public async delete(
        path: string,
        body?: string,
        headers?: HeadersInit,
    ): Promise<Response> {
        return await this.do('DELETE', path, body, headers);
    }

    /**
   * do sends an HTTP request and returns an HTTP response as configured on the client.
   * @param method holds http method type
   * @param path
   * @param body serialized JSON
   * @param headers holds request header
   */
    private async do(
        method: string,
        path: string,
        body?: string,
        headers: HeadersInit = { 'Content-Type': 'application/json' }
    ): Promise<Response> {
        const request: RequestInit = {
            method,
            body,
            headers,
        };

        return await fetch(path, request);
    }
}
