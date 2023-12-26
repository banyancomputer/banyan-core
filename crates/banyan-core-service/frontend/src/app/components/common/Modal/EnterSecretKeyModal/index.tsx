import { useIntl } from 'react-intl';
import { useForm } from 'react-hook-form';

import { Input } from '@components/common/Input';
import { SubmitButton } from '@components/common/SubmitButton';

import { Bolt } from '@static/images/common';
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

    const confirm = async() => {
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
                <Bolt width="24px" height="24px" />
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
            <SubmitButton
                text={`${messages.confirm}`}
                disabled={!keyphrase}
                className="py-2.5"
            />
        </form >
    );
};
