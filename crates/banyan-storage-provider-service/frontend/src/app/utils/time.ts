export const getDateLabel = (date: Date, year: boolean = true) => {
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

    return `${months[date.getMonth()]} ${date.getDate()} ${year ? `, ${date.getFullYear()}`: ''}`;
};

export const getDealDateLabel = (date: Date) => {

    return date.toLocaleDateString('en');
};
export const getNotiifcationDateLabel = (date: Date) => {

    return `${date.getHours()}.${date.getMinutes()} ${date.toLocaleDateString('en')}`;
};
