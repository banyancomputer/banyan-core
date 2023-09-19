import React, { ReactElement } from 'react';
import {
    AiOutlineFile, AiOutlineFileExcel, AiOutlineFileGif,
    AiOutlineFileImage, AiOutlineFileJpg, AiOutlineFilePdf,
    AiOutlineFilePpt, AiOutlineFileText, AiOutlineFileWord,
    AiOutlineFileZip,
} from 'react-icons/ai';
import { PiFolderNotchBold } from 'react-icons/pi';

export const FileIcon: React.FC<{ fileName: string; className?: string }> = ({ fileName, className }) => {
    const fileTypeMapper: Record<string, ReactElement> = {
        'txt': <AiOutlineFileText size="24px" fill="#7D89B0" />,
        'pdf': <AiOutlineFilePdf size="24px" fill="#7D89B0" />,
        'doc': <AiOutlineFileWord size="24px" fill="#7D89B0" />,
        'docx': <AiOutlineFileWord size="24px" fill="#7D89B0" />,
        'ppt': <AiOutlineFilePpt size="24px" fill="#7D89B0" />,
        'pptx': <AiOutlineFilePpt size="24px" fill="#7D89B0" />,
        'xls': <AiOutlineFileExcel size="24px" fill="#7D89B0" />,
        'xlsx': <AiOutlineFileExcel size="24px" fill="#7D89B0" />,
        'jpg': <AiOutlineFileJpg size="24px" fill="#7D89B0" />,
        'jpeg': <AiOutlineFileJpg size="24px" fill="#7D89B0" />,
        'png': <AiOutlineFileImage size="24px" fill="#7D89B0" />,
        'gif': <AiOutlineFileGif size="24px" fill="#7D89B0" />,
        'zip': <AiOutlineFileZip size="24px" fill="#7D89B0" />,
        'rar': <AiOutlineFileZip size="24px" fill="#7D89B0" />,
    };

    return (
        <div className={className}>
            {
                fileName.split('.')[1] ?
                    fileTypeMapper[fileName.split('.')[1]] || <AiOutlineFile size="24px" fill="#7D89B0" />
                    :
                    <PiFolderNotchBold size="24px" fill="#7D89B0" />
            }
        </div>
    );
};
