import { useState } from 'react';
import { useIntl } from 'react-intl';

import { TrashTable } from '@/app/components/Trash/TrashTable';
import { EmptyTrashModal } from '@/app/components/common/Modal/EmptyTrashModal';
import { Fallback } from '@/app/components/common/Fallback';

import { useTomb } from '@/app/contexts/tomb';
import { useModal } from '@/app/contexts/modals';

import { ArrowDown, Close, EmptyIcon, ErrorBannerIcon } from '@static/images/common';

const Trash = () => {
    const { trash } = useTomb();
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
            <Fallback shouldRender>
                {trash?.files.length ?
                    <>
                        {isLabelVisible &&
                            <div className="relative mb-5 flex items-start gap-3 p-4 border-1 rounded-md bg-gray-200 border-gray-600">
                                <div><ErrorBannerIcon /></div>
                                <div className="text-xs">
                                    <h6 className="font-semibold">{`${messages.trashIsFull}`}</h6>
                                    <p className="mt-1 ">{`${messages.clickToEmptyTrash}`}.</p>
                                    <button className="flex items-center gap-2 mt-3 font-semibold" onClick={emptyTrash}>
                                        {`${messages.emptyTrash}`}
                                        <span className="-rotate-90">
                                            <ArrowDown width="20px" height="20px" />
                                        </span>
                                    </button>
                                    <button className="absolute right-4 top-4" onClick={closeLabel}>
                                        <Close />
                                    </button>
                                </div>
                            </div>
                        }
                        <TrashTable bucket={trash} />
                    </>
                    :
                    <div className="h-full flex flex-col items-center justify-center saturate-0">
                        <EmptyIcon />
                        <p className="mt-4">{`${messages.trashIsEmpty}`}</p>
                    </div>
                }
            </Fallback>
        </section>
    );
};

export default Trash;
