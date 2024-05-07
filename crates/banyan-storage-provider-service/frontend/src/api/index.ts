import { HttpClient } from '@/api/http/client';

/*
**
 * ErrorUnauthorized is a custom error type which indicates that the client request has not been
 * completed because it lacks valid authentication credentials for the requested resource.
 */
export class UnauthorizedError extends Error {
    /** Error message while unauthorized */
    public constructor(message = 'Unauthorized') {
        super(message);
    }
}
/**
 * BadRequestError is a custom error type which indicates that the server cannot or
 * will not process the request due to something that is perceived to be a client error.
 */
export class BadRequestError extends Error {
    /** Error message while bad request */
    public constructor(message = 'bad request') {
        super(message);
    }
}
/**
 * ForbiddenError is a custom error type which indicates that the resource is not allowed.
 */
export class ForbiddenError extends Error {
    /** Error message while resource is not allowed */
    public constructor(message = 'not allowed') {
        super(message);
    }
}
/**
 * NotFoundError is a custom error type which indicates that the server can't find the requested resource.
 */
export class NotFoundError extends Error {
    /** Error message while not found request */
    public constructor(message = 'not found') {
        super(message);
    }
}

/**
 * InternalError is a custom error type which indicates that the server encountered an unexpected condition
 * that prevented it from fulfilling the request.
 */
export class InternalError extends Error {
    /** Error message for internal server error */
    public constructor(message = 'internal server error') {
        super(message);
    }
}

const BAD_REQUEST_ERROR = 400;
const UNAUTHORISED_ERROR = 401;
const FORBIDDEN_ERROR = 403;
const NOT_FOUND_ERROR = 404;
const INTERNAL_ERROR = 500;

/**
 * APIClient is base client that holds http client and error handler.
 */
export class APIClient {
    protected readonly http: HttpClient = new HttpClient();
    protected readonly ROOT_PATH = `${process.env.API_URL || ''}`;

    /**
         * handles error due to response code.
         * @param response - response from server.
         *
         * @throws {@link NotFoundError}
         * This exception is thrown if the input is not a valid ISBN number.
         *
         * @throws {@link UnauthorizedError}
         * Thrown if the ISBN number is valid, but no such book exists in the catalog.
         *
         * @throws {@link InternalError}
         * Thrown if the ISBN number is valid, but no such book exists in the catalog.
         *
         * @private
         */
    /* eslint-disable */
    protected async handleError(response: Response): Promise<Error> {
        const error = await response.json();
        switch (response.status) {
            case BAD_REQUEST_ERROR:
                return new BadRequestError(error.message);
            case FORBIDDEN_ERROR:
                return new ForbiddenError(error.message);
            case NOT_FOUND_ERROR:
                return new NotFoundError(error.message);
            case UNAUTHORISED_ERROR: {
                return new UnauthorizedError(error.message);
            }
            case INTERNAL_ERROR:
            default:
                return new InternalError(error.message);
        }
    }
}
