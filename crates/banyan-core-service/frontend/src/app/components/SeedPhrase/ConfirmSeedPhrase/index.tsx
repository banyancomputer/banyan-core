import React, { useState } from 'react';
import { SeedPhraseInput } from '@components/common/SeedPhraseInput';
import { PrimaryButton } from '../../common/PrimaryButton';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useNavigate } from 'react-router-dom';

export const ConfirmSeedPhrase: React.FC<{
    seedPhrase: string[],
}> = ({ seedPhrase }) => {
    const navigate = useNavigate();
    const [seedPhraseCopy, setSeedPhraseCopy] = useState(seedPhrase.map((word, index) => index % 4 ? word : ''));

    const cofirmSeedPhrase = async () => {
        if (seedPhrase.join('') !== seedPhraseCopy.join('')) {
            ToastNotifications.error('Seed phrase do not match');
            return;
        };

        navigate('/');
    };

    return (
        <div className="flex flex-col gap-4 w-[428px]">
            <div className="flex flex-col gap-2 items-center">
                <h2 className="text-lg font-semibold">Confirm your seed phrase</h2>
                <p className="text-xs">Enter your Seed Phrase to reset your encryption key</p>
            </div>
            <SeedPhraseInput
                seedPhrase={seedPhraseCopy}
                onChange={setSeedPhraseCopy}
            />
            <PrimaryButton
                text="Сonfirm"
                action={cofirmSeedPhrase}
            />
        </div>
    )
}
