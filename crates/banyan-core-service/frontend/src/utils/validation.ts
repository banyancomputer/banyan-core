
/** Returns validation error message for react-hook-form library */
export const validateKeyphrase = (message: string) => {
    return (keyphrase: string) => {
        const keyphraseRegexp = new RegExp(/^.{8,}$/);
        return keyphraseRegexp.test(keyphrase)? undefined : message;
    }
}