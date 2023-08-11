export const getDateLabel = (timestapm: string) => {
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    const date = new Date(timestapm);

    return `${months[date.getMonth()]} ${date.getDay()}, ${date.getFullYear()}`;
};
