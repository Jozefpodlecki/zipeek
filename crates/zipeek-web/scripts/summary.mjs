import {Chart, DoughnutController, ArcElement, BarController, BarElement, CategoryScale, LinearScale} from 'chart.js';

Chart.register(DoughnutController, ArcElement, BarController, BarElement, CategoryScale, LinearScale);

export function createChart(canvas, configuration) {
    return new Chart(canvas, configuration);
}
