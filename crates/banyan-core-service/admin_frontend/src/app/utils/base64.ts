export function stringToBase64(string: string) {
	return btoa(string);
}

export function base64ToString(string: string) {
	return atob(string);
}
