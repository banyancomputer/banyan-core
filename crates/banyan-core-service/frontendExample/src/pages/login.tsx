import { useEffect, useState } from 'react';
import { NextPageWithLayout } from '@/pages/page';
import { signIn, useSession } from 'next-auth/react';
import { Button } from '@chakra-ui/react';
import Router from 'next/router';
import BaseLayout from '@/layouts/base/BaseLayout';

const Login: NextPageWithLayout = ({}) => {
	const { status } = useSession();
	const [error, setError] = useState('');

	// Redirect to home page if user is logged in
	useEffect(() => {
		if (status) {
			Router.push('/').then(() => window.scrollTo(0, 0));
		}
	}, [status]);

	const handleLoginWithProvider = (provider: any) => () => {
		signIn(provider)
			.then(() => {
				Router.push('/').then(() => window.scrollTo(0, 0));
			})
			.catch((err) => {
				setError(err.message);
			});
	};

	return (
		<div>
			<div className="text-6xl font-semibold align-left mb-2">Log in</div>
			{error && (
				// Error when login fails
				<label htmlFor="registration" className="label">
					<span className="text-xxs !p-0 text-error text-left">
						There was an issue logging you in. Please try again.
					</span>
				</label>
			)}
			<div className="flex items-center mt-4">
				<Button
					colorScheme="blue"
					variant="solid"
					ml={4}
					w={40}
					onClick={handleLoginWithProvider('google')}
				>
					Log in with Google
				</Button>
			</div>
		</div>
	);
};

export default Login;

Login.getLayout = (page) => {
	return <BaseLayout>{page}</BaseLayout>;
};
