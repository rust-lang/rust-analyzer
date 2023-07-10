// @ts-nocheck
export function showCrateDependencyGraph(dot): void {
    const graph = d3
        .select("#graph")
        .graphviz({ useWorker: false, useSharedWorker: false })
        .fit(true)
        .zoomScaleExtent([0.1, Infinity])
        .renderDot(dot);

    d3.select(window).on("click", (event) => {
        if (event.ctrlKey) {
            graph.resetZoom(d3.transition().duration(100));
        }
    });
    d3.select(window).on("copy", (event) => {
        event.clipboardData.setData("text/plain", dot);
        event.preventDefault();
    });
}
