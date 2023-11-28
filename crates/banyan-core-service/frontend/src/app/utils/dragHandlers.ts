export const preventDefaultDragAction = (event: React.DragEvent<HTMLElement>) => {
    event.preventDefault();
    event.stopPropagation();
};