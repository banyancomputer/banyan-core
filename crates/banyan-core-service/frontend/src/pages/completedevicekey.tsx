import React, { useEffect, useState } from 'react';
import { useIntl } from 'react-intl';
import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from './page';
import getServerSideProps from '@/utils/session';
import { useTomb } from '@/contexts/tomb';
import { useModal } from '@/contexts/modals';
import { useSearchParams } from 'next/navigation';
import { b64UrlDecode } from '@/utils/b64';
import { prettyFingerprintApiKeySpki, publicPemWrap } from '@/utils';
import { useSession } from 'next-auth/react';
import { ClientApi } from '@/lib/api/auth';
import { DeviceApiKey } from '@/lib/interfaces';

export { getServerSideProps };

const DeviceKeyApproval: NextPageWithLayout = () => {
    const api = new ClientApi();
    const { openModal } = useModal();
    const { messages } = useIntl();
    const { data: session } = useSession();
    const { completeDeviceKeyRegistration } = useTomb();
    const searchParams = useSearchParams();
    const urlSpki = searchParams.get('spki')!;
    const spki = b64UrlDecode(urlSpki as string);
    const pem = publicPemWrap(spki);
    const [fingerprint, setFingerprint] = useState("");

    useEffect(() => {
        const getFingerprint = async () => {
            let fingerprint: string = await prettyFingerprintApiKeySpki(spki);
            setFingerprint(fingerprint);
        }
        getFingerprint();
    }, []);

    // Perform all functions required to complete 
    const completeRegistration = async () => {
        try {
            let keys: DeviceApiKey[] = await api.readDeviceApiKeys();
            if (keys.some(key => key.fingerprint == fingerprint)) {
                console.log("key already registered; sending completion signal");
                await completeDeviceKeyRegistration(fingerprint);
            }
            else {
                console.log("failed to find an existing key with that fingerprint; adding " + fingerprint);
                await api.registerDeviceApiKey(pem);
                await completeDeviceKeyRegistration(fingerprint);
            }

            console.log("finished device key completion");
            alert("successfully authorized device!");
        } catch (error: any) { 
            alert("failed to authorize new device!");
            console.log("error: " + error);
        }
    };

    return (
        <section className="py-9 px-4" id="buckets">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semixld">
                    Approve new device key?
                </h2>
                <div className="w-modal flex flex-col gap-8" >
                    <div>
                        <h4 className="text-m font-semibold">{`${messages.approveAccess}`}</h4>
                        <p className="mt-2 text-text-600">
                            {`${messages.wantToApproveAccess}?`}
                        </p>

                        <h4 className="text-m font-semibold">Fingerprint:</h4>
                        <p className="mt-2 text-text-600">{`${fingerprint}`}</p>

                        <h4 className="text-m font-semibold">PEM:</h4>
                        <p className="mt-2 text-text-600">{`${spki}`}</p>
                    </div>
                    <div className="mt-3 flex items-center gap-3 text-xs" >
                        <button
                            className="btn-secondary w-1/2 py-3 px-4"
                        // onClick={closeModal}
                        >
                            {`${messages.cancel}`}
                        </button>
                        <button
                            className="btn-primary w-1/2 py-3 px-4"
                            onClick={completeRegistration}
                        >{`${messages.approveAccess}`}</button>
                    </div>
                </div>

            </div>
        </section>
    );
};

export default DeviceKeyApproval;
DeviceKeyApproval.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
