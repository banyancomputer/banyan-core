/** Checks filelist and added duplication identificator if name already exists if filelist. */
export const handleNameDuplication = (fileName: string, fileList: string[]): string => {
    let count = 1;
    let newFileName = fileName;

    while (fileList.includes(newFileName)) {
        const dotIndex = fileName.lastIndexOf('.');
        const extension = dotIndex !== -1 ? fileName.slice(dotIndex) : '';
        const baseName = dotIndex !== -1 ? fileName.slice(0, dotIndex) : fileName;

        newFileName = `${baseName} (${count})${extension}`;
        count++;
    }

    return newFileName;
};