export const getUSDAmount = (amount: string) => {
    return (+amount / 10000).toFixed(2);
}