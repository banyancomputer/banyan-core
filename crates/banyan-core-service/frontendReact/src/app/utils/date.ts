export const getDateLabel = (timestapm: number, year: boolean = true) => {
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    const date = new Date(timestapm * 1000);

    return `${months[date.getMonth()]} ${date.getDay()} ${year ? `, ${date.getFullYear()}`: ''}`;
};

export const getTime = (timestapm: number, year: boolean = true) => new Date(timestapm).toLocaleTimeString('en');
