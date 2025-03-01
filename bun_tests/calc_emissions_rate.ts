const calculateAmount = (initialAmount: number, rate: number, targetDate: string) => {
    const startDate = new Date('2025-02-28'); // Static start date
    const secondsIn30Days = 30 * 24 * 60 * 60;

    // Convert dates to seconds
    const targetTime = new Date(targetDate).getTime() / 1000;
    const startTime = startDate.getTime() / 1000;

    // Calculate elapsed time and number of 30-day periods
    const elapsedTime = targetTime - startTime;
    const periods = Math.floor(elapsedTime / secondsIn30Days);

    // Calculate the amount after the elapsed periods
    const amount = initialAmount * Math.pow((1 - rate), periods);
    return amount;
};

// Usage:
const initialAmount = 1000;
const rate = 0.05;
const targetDate = '2025-04-30'; // Change this to any date you want

const amountAtDate = calculateAmount(initialAmount, rate, targetDate);
console.log(`Amount on ${targetDate}: ${amountAtDate.toFixed(2)}`);
