import { Action } from '@components/Bucket/Files/BucketTable/FileActions';
import { Rename, Versions } from '@app/static/images/common';
import { useAppSelector } from '@app/store';
import { Bucket, BucketSnapshot } from '@/app/types/bucket';
import { useModal } from '@app/contexts/modals';
import { RenameSnapshotModal } from '@components/common/Modal/RenameSnapshotModal';

export const SnapshotActions: React.FC<{ bucket: Bucket, snapshot: BucketSnapshot }> = ({ bucket, snapshot }) => {
    const { openModal } = useModal();
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.snapshots.table.snapshotActions);

    const rename = () => {
        openModal(<RenameSnapshotModal bucket={bucket} snapshot={snapshot} />);
    };

    /** TODO: implement when backend will be ready */
    const restore = () => { };

    const actions: Action[] = [
        new Action(messages.rename, <Rename width="18px" height="18px" />, rename),
        new Action(messages.restore, <Versions width="18px" height="18px" />, restore),
    ];

    return (
        <div className="w-48 right-8 text-xs font-medium bg-bucket-actionsBackground rounded-xl shadow-md z-10 select-none text-bucket-actionsText overflow-hidden">
            {actions.map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 transition-all hover:bg-hover"
                    onClick={action.value}
                    id="action"
                >
                    {action.icon}
                    {action.label}
                </div>
            )}
        </div>
    )
}
