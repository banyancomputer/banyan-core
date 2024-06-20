import { AudioFileIcon, CommonFileIcon, ImageFileIcon, PdfFileIcon, VideoFileIcon } from '../static/images/common';

const SUPPORTED_AUDIO = { type: 'audio', mimeTypes: ['audio/mpeg', 'audio/ogg', 'audio/wav'], icon: AudioFileIcon };
const SUPPORTED_DOCUMENT = { type: 'document', mimeTypes:['application/pdf'], icon: PdfFileIcon };
const SUPPORTED_IMAGE = { type: 'image', mimeTypes: ['image/jpeg', 'image/png', 'image/gif', 'image/bmp', 'image/svg+xml', 'image/webp'], icon: ImageFileIcon };
const SUPPORTED_SPREADSHEET = { type: 'spreadsheet', mimeTypes:['text/csv'], icon: CommonFileIcon };
const SUPPORTED_VIDEO = { type: 'video', mimeTypes:['video/mp4', 'video/webm', 'video/ogg'], icon: VideoFileIcon };

export const SUPPORTED_FILE_TYPES = [SUPPORTED_AUDIO, SUPPORTED_DOCUMENT, SUPPORTED_IMAGE, SUPPORTED_SPREADSHEET, SUPPORTED_VIDEO];
