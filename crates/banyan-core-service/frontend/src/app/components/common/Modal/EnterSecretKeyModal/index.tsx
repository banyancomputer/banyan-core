import { useForm } from 'react-hook-form';

import { Input } from '@components/common/Input';
import { PrimaryButton } from '@components/common/PrimaryButton';

import { Bolt } from '@static/images/common';
import { useKeystore } from '@/app/contexts/keystore';
import { useModal } from '@/app/contexts/modals';
import { validateKeyphrase } from '@/app/utils/validation';
import { useAppSelector } from '@/app/store';

export const EnterSecretKeyModal = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.enterSecretKey);
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
            setError('keyphrase', { message: `${messages.sectretKeyError}` });
        };
    };

    return (
        <form
            className="w-modal flex flex-col gap-8"
            onSubmit={handleSubmit(confirm)}
        >
            <div>
                <h4 className="text-m font-semibold">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}`}
                </p>
            </div>
            <Input
                type="password"
                label={`${messages.secretKey}`}
                placeholder={`${messages.enterSecretKey}`}
                error={errors.keyphrase?.message}
                register={register('keyphrase', {
                    required: `${messages.enterSecretKey}`,
                    validate: validateKeyphrase(`${messages.keyRequirements}`),
                })}
            />
            <PrimaryButton
                text={`${messages.confirm}`}
                disabled={!keyphrase}
                className="py-2.5"
            />
        </form >
    );
};
