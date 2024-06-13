
import { useEffect } from 'react';

import { KeyManagementTable } from '@components/Account/ManageKeys/KeyManagementTable';
import { Fallback } from '@components/common/Fallback';

import { ToastNotifications } from '@/app/utils/toastNotifications';
//<<<<<<< HEAD
//import { PrimaryButton } from '../../common/PrimaryButton';
//import { PlusBold } from '@/app/static/images/common';
//import { useAppDispatch } from '@/app/store';
//import { openModal } from '@/app/store/modals/slice';
//import { CreateAccessKey } from '../../common/Modal/CreateAccessKey';
//||||||| 919bd72e
//=======
import { useAppDispatch, useAppSelector } from '@store/index';
import { getBucketsKeys } from '@store/tomb/actions';
//>>>>>>> main

const ManageKeys = () => {
	const dispatch = useAppDispatch();
	//<<<<<<< HEAD
	//	const { buckets, areAccessKeysLoading, getUserAccessKeys, userAccessKeys, tomb } = useTomb();
	//
	//	const addKey = () => {
	//		dispatch(openModal({ content: <CreateAccessKey /> }))
	//	};
	//||||||| 919bd72e
	//    const { buckets, areBucketsLoading, tomb, getBucketsKeys } = useTomb();
	//=======
	const { buckets, isLoading, tomb, } = useAppSelector(state => state.tomb);
	//>>>>>>> main

	useEffect(() => {
		if (!tomb) { return; }

		const getAccess = async () => {
			try {
				//<<<<<<< HEAD
				//				await getUserAccessKeys();
				//||||||| 919bd72e
				//                await getBucketsKeys();
				//=======
				await dispatch(getBucketsKeys());
				//>>>>>>> main
			} catch (error: any) {
				ToastNotifications.error('Failed to get user access keys', 'Try again', getAccess)
			};
		};

		getAccess();
	}, [buckets.length, tomb]);

	return (
		//<<<<<<< HEAD
		//		<div className="flex flex-grow flex-col items-start gap-5 p-6">
		//			<Fallback shouldRender={!areAccessKeysLoading}>
		//				<PrimaryButton
		//					text="Add Key"
		//					icon={<PlusBold />}
		//					action={addKey}
		//				/>
		//				<KeyManagementTable userAccessKeys={userAccessKeys} />
		//||||||| 919bd72e
		//        <div className="flex flex-grow flex-col gap-5 p-6">
		//					<Fallback shouldRender={!areBucketsLoading}>
		//						<KeyManagementTable buckets={buckets} />
		//=======
		<div className="flex flex-grow flex-col gap-5 p-6">
			<Fallback shouldRender={!isLoading}>
				<KeyManagementTable buckets={buckets} />
			</Fallback>
		</div>
	);
};

export default ManageKeys;
