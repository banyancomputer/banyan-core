import React, { useState } from 'react';
import { HiOutlineLightningBolt } from 'react-icons/hi';
import { useIntl } from 'react-intl';
import { useForm } from 'react-hook-form';

import { Input } from '../../Input';

import { useKeystore } from '@/app/contexts/keystore';
import { useModal } from '@/app/contexts/modals';
import { validateKeyphrase } from '@/app/utils/validation';

export const EnterSecretKeyModal = () => {
	const { messages } = useIntl();
	const { initializeKeystore } = useKeystore();
	const { closeModal } = useModal();
	const {
		formState: { errors },
		handleSubmit,
		register,
		setError,
		watch,
	} = useForm({
		mode: 'onTouched',
		values: { keyphrase: '' },
	});
	const { keyphrase } = watch();

	const confirm = async () => {
		try {
			await initializeKeystore(keyphrase);
			closeModal();
		} catch (error: any) {
			/** TODO: rework when error message from tomb will be more specific. */
			setError('keyphrase', { message: `${messages.wrongSecretKey}` });
		};
	};

	return (
		<form
			className="w-modal flex flex-col gap-8"
			onSubmit={handleSubmit(confirm)}
		>
			<span className="p-3 w-min rounded-full bg-button-disabled">
				<HiOutlineLightningBolt size="24px" />
			</span>
			<div>
				<h4 className="text-m font-semibold">{`${messages.inputSecretKey}`}</h4>
				<p className="mt-2 text-text-600">
					{`${messages.enterSecretKey}`}
				</p>
			</div>
			<Input
				type="password"
				label={`${messages.secretKey}`}
				placeholder={`${messages.enterPassphrase}`}
				error={errors.keyphrase?.message}
				register={register('keyphrase', {
					required: `${messages.enterPassphrase}`,
					validate: validateKeyphrase(`${messages.keyRequirements}`),
				})}
			/>
			<button
				type="submit"
				className="btn-primary flex-grow py-2.5 px-4"
				disabled={!keyphrase}
			>
				{`${messages.confirm}`}
			</button>
		</form >
	);
};