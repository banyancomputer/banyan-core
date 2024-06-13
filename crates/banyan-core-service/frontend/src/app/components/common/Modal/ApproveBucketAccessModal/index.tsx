import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { closeModal } from '@store/modals/slice';
//<<<<<<< HEAD
import { useTomb } from '@/app/contexts/tomb';
import { Bucket } from '@/app/types/bucket';
//||||||| 919bd72e
//import { useTomb } from '@/app/contexts/tomb';
//import { Bucket, BucketKey } from '@/app/types/bucket';
//=======
//import { Bucket, BucketKey } from '@/app/types/bucket';
//>>>>>>> main
import { ToastNotifications } from '@/app/utils/toastNotifications';
//<<<<<<< HEAD
//import { useAppDispatch, useAppSelector } from '@/app/store';
import { UserAccessKey } from '@/app/types/userAccessKeys';
//||||||| 919bd72e
//import { useAppDispatch, useAppSelector } from '@/app/store';
//=======
import { useAppDispatch, useAppSelector } from '@store/index';
import { approveBucketAccess } from '@store/tomb/actions';
import { unwrapResult } from '@reduxjs/toolkit';
//>>>>>>> main

export const ApproveBucketAccessModal: React.FC<{ bucket: Bucket; accessKey: UserAccessKey }> = ({ bucket, accessKey }) => {
	const dispatch = useAppDispatch();
	const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.approveBucketAccess);

	const cancel = () => {
		dispatch(closeModal());
	};

	const approveAccess = async () => {
		try {
			//<<<<<<< HEAD
			await approveBucketAccess(bucket, accessKey.publicKey);
			//||||||| 919bd72e
			//            await approveBucketAccess(bucket, bucketKey.id);
			//=======
			//            unwrapResult(await dispatch(approveBucketAccess({ bucket, bucketKeyId: bucketKey.id })));
			//>>>>>>> main
			cancel();
		} catch (error: any) {
			ToastNotifications.error('Something went wrong', `${messages.tryAgain}`, approveAccess);
		}
	};

	return (
		<div className="w-modal flex flex-col gap-8" >
			<div>
				<h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
				<p className="mt-2 text-text-600">
					{`${messages.subtitle}?`}
				</p>
			</div>
			<div className="mt-3 flex items-center justify-end gap-3 text-xs" >
				<SecondaryButton
					action={cancel}
					text={`${messages.cancel}`}
				/>
				<PrimaryButton
					text={`${messages.approveAccess}`}
					action={approveAccess}
				/>
			</div>
		</div>
	);
};
