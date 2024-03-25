import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';

import { Input } from '@components/common/Input';
import { PrimaryButton } from '@components/common/PrimaryButton';

import { useAppSelector } from '@app/store';
import { useKeystore } from '@app/contexts/keystore';

const EnterEncryptionKey = () => {
    const messages = useAppSelector(state => state.locales.messages.pages.enterEncryptionKey);
    const { initializeKeystore } = useKeystore();
    const navigate = useNavigate();
    const {
        formState: { errors },
        handleSubmit,
        register,
        setError,
        watch,
    } = useForm({
        mode: 'all',
        values: { keyphrase: '' },
    });
    const { keyphrase } = watch();
    const isDataCorrect = !Object.keys(errors).length && keyphrase.length >= 8;

    const confirm = async () => {
        try {
            await initializeKeystore(keyphrase);
            navigate('/');
        } catch (error: any) {
            setError('keyphrase', { message: `${messages.secretKeyError}` });
        };
    };

    return (
        <form
            className="flex flex-col gap-4 w-[428px] max-w-[428px]"
            onSubmit={handleSubmit(confirm)}
        >
            <h2 className="text-[32px] font-semibold">{messages.title}</h2>
            <Input
                type="password"
                label={`${messages.encryptionKey}`}
                placeholder={`${messages.encryptionKeyPlaceholder}`}
                error={errors.keyphrase?.message}
                labelClassName="font-semibold"
                register={register('keyphrase', {
                    required: `${messages.encryptionKeyPlaceholder}`,
                })}
            />
            {/* <div className="flex items-center gap-1.5 text-[13px] font-medium whitespace-nowrap text-button-primary ">
                {`${messages.forgotEncryptionKey}?`}
                <button className="underline font-semibold hover:text-button-primaryHover">
                    {messages.resetKey}
                </button>
            </div> */}
            <PrimaryButton
                action={confirm}
                text={messages.continue}
                disabled={!isDataCorrect}
            />
        </form>
    )
};

export default EnterEncryptionKey;
