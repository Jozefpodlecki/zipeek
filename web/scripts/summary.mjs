import {Chart, DoughnutController, ArcElement} from 'chart.js';

Chart.register(DoughnutController, ArcElement);

export function createSummary(canvas, count, correct, total) {
    const media = matchMedia('(prefers-reduced-motion: reduce)');
    const chart = new Chart(canvas, {
        type: 'doughnut',
        data: {datasets: [{data: [correct, total - correct], backgroundColor: ['#2563eb', '#94a3b8'], borderWidth: 0}]},
        options: {
            responsive: true,
            maintainAspectRatio: false,
            cutout: '72%',
            events: [],
            animation: media.matches ? false : {duration: 300, easing: 'linear'},
            plugins: {legend: false, tooltip: false}
        }
    });
    let frame;
    let disposed = false;
    const paint = value => { if (count.firstChild) count.firstChild.nodeValue = String(value); };
    const finish = () => {
        cancelAnimationFrame(frame);
        paint(correct);
        chart.options.animation = false;
        chart.stop();
        chart.update('none');
        canvas.dataset.chartAnimation = '0';
    };
    const changed = () => {if (media.matches && !disposed) finish();};
    media.addEventListener('change', changed);
    if (!media.matches) {
        const start = performance.now();
        const step = now => {
            if (disposed) return;
            const fraction = Math.min(1, (now - start) / 300);
            paint(Math.round(correct * fraction));
            if (fraction < 1) frame = requestAnimationFrame(step);
        };
        paint(0);
        frame = requestAnimationFrame(step);
    }
    canvas.dataset.summaryChart = 'ready';
    canvas.dataset.chartAnimation = media.matches ? '0' : '300';
    return {destroy() {
        if (disposed) return;
        disposed = true;
        cancelAnimationFrame(frame);
        media.removeEventListener('change', changed);
        chart.destroy();
        canvas.dataset.summaryChart = 'destroyed';
    }};
}

export function destroySummary(handle) {handle.destroy();}
