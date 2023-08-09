/** Will be used for modals and context menus to handle closing */
export const otsideClickHandler = (
	ref: HTMLDivElement,
	onchange: (visible: boolean) => void
) =>
	function (this: Document, event: MouseEvent) {
		if (!ref) {
			return;
		}

		if (!ref.contains(event.target as Node)) {
			onchange(false);
		}
	};
