import { BrowserObject, Bucket } from "@app/types/bucket";

export const preventDefaultDragAction = (event: React.DragEvent<HTMLElement>) => {
    event.preventDefault();
    event.stopPropagation();
};

/** Sets drag data, removes default drag preview. */
export const handleDragStart = async (
    event: React.DragEvent<HTMLDivElement>,
    item: BrowserObject,
    setIsDragging: React.Dispatch<React.SetStateAction<boolean>>,
    path: string[],
    ) => {
    event.stopPropagation()
    event.dataTransfer.setData('browserObject', JSON.stringify({ item, path }));
    event.dataTransfer.setDragImage(new Image(), 0, 0);
    setIsDragging(true);
};

/** Gets drag preview element, and sets its position according to cursors */
export const handleDrag = (event: React.DragEvent<HTMLDivElement>, name: string) => {
    const element = document.getElementById(`dragging-preview-${name}`);
    if(element) {
        element.style.top = `${event?.clientY}px`;
        element.style.left = `${event?.clientX}px`;
    }
};

/** Sets dragging state to false, ,which will hide drag preview. */
export const handleDragEnd = async(
    setIsDragging: React.Dispatch<React.SetStateAction<boolean>>,
    updateFolderState: (path: string[], folder: BrowserObject, bucket: Bucket) => void,
    updateBucketState: (path: string[]) => void,
    path: string[],
    folder: BrowserObject | undefined,
    bucket: Bucket
    ) => {
    setIsDragging(false);
    folder?
    await updateFolderState(path, folder, bucket)
    :
    await updateBucketState(path);
};