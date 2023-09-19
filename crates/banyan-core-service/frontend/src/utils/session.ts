import { Session, getServerSession } from 'next-auth';
import { authOptions } from '../pages/api/auth/[...nextauth]';
import { AccountFactory, EscrowedDeviceFactory } from '@/lib/db';

export default async function getServerSideProps(context: any) {
    // If the user has a session, serve the page
    const session: Session | null = await getServerSession(
        // @ts-ignore
        context.req,
        context.res,
        authOptions
    );

    if (!session) {
        return {
            redirect: {
                destination: '/login',
                permanent: false,
            },
        };
    }

    return {
        props: {},
    };
}
