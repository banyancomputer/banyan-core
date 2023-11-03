
/** Returns validation error message for react-hook-form library */
export const validateKeyphrase = (message: string) => (keyphrase: string) => {
    const keyphraseRegexp = new RegExp(/^.{8,}$/);

    return keyphraseRegexp.test(keyphrase)? undefined : message;
};
