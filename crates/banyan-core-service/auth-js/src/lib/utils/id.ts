export const splitProviderId = (providerId: string): [string, string] => {
	const [provider, providerAccountId] = providerId.split(':');
	return [provider, providerAccountId];
};

export const joinProviderId = (
	provider: string,
	providerAccountId: string
): string => {
	return `${provider}:${providerAccountId}`;
};
