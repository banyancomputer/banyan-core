/* Set value to localStorage */
export const setLocalStorageItem = (item: string, value: string) =>
    window?.localStorage.setItem(item, value);

/* Get value from localStorage */
export const getLocalStorageItem = (item: string) =>
    window?.localStorage.getItem(item);

/* Remove item from localStorage */
export const removeLocalStorageItem = (item: string) =>
    window?.localStorage.removeItem(item);

