/** Will be used for modals and context menus to handle closing */
export const popupClickHandler = (ref: HTMLDivElement, onchange:(visible: boolean) => void) =>
    function(this: Document, event: MouseEvent) {
        console.log('ref', !ref.contains(event.target as Node));
        
        if (!ref) {
            return;
        };

        if (!ref.contains(event.target as Node)) {
            onchange(false);
        };
    };
