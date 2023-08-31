import React, { useState } from 'react';
import { useIntl } from 'react-intl';
import { FiAlertCircle, FiArrowRight } from 'react-icons/fi';
import { IoMdClose } from 'react-icons/io';
import Image from 'next/image';

import { NextPageWithLayout } from './page';
import BaseLayout from '@/layouts/BaseLayout';
import { useTomb } from '@/contexts/tomb';
import { TrashTable } from '@/components/Trash/TrashTable';
import { useModal } from '@/contexts/modals';
import { EmptyTrashModal } from '@/components/common/Modal/EmptyTrashModal';
import { Fallback } from '@/components/common/Fallback';

import emptyTrashIcon from '@static/images/trash/emptyTrashIcon.png';

import getServerSideProps from '@/utils/session';

export { getServerSideProps };

const Trash: NextPageWithLayout = () => {
    const { trash, isTrashLoading } = useTomb();
    const { messages } = useIntl();
    const { openModal } = useModal();
    const [isLabelVisible, setIsLabelVisible] = useState(true);

    const closeLabel = () => {
        setIsLabelVisible(false);
    };

    const emptyTrash = () => {
        openModal(<EmptyTrashModal />);
    };

    return (
        <section className="py-9 px-4 h-full">
            <div className="mb-4">
                <h2 className="text-xl font-semibold">
                    {`${messages.trash}`}
                </h2>
            </div>
            <Fallback shouldRender={!isTrashLoading}>
                {trash.files.length ?
                    <>
                        {isLabelVisible &&
                            <div className="relative mb-5 flex items-start gap-3 p-4 border-1 rounded-lg bg-gray-200 border-gray-600">
                                <div><FiAlertCircle size="20px" /></div>
                                <div className="text-xs">
                                    <h6 className="font-semibold">{`${messages.trashIsFull}`}</h6>
                                    <p className="mt-1 ">{`${messages.clickToEmptyTrash}`}.</p>
                                    <button className="flex items-center gap-2 mt-3 font-semibold" onClick={emptyTrash}>
                                        {`${messages.emptyTrash}`}
                                        <FiArrowRight size="20px" />
                                    </button>
                                    <button className="absolute right-4 top-4" onClick={closeLabel}>
                                        <IoMdClose size="20px" fill="#4A5578" />
                                    </button>
                                </div>
                            </div>
                        }
                        <TrashTable bucket={trash} />
                    </>
                    :
                    <div className="h-full flex flex-col items-center justify-center saturate-0">
                        <Image src={emptyTrashIcon} alt="EmptyTrashIcon" />
                        <p className="mt-4">{`${messages.trashIsEmpty}`}</p>
                    </div>
                }
            </Fallback>
        </section>
    );
};

export default Trash;

Trash.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
