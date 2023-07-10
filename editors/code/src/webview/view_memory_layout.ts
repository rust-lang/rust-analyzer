// @ts-nocheck

export function showMemoryLayout(data): void {
    if (!(data && data.nodes.length)) {
        document.body.innerText = "Not Available";
        return;
    }

    data.nodes.map((n) => {
        n.typename = n.typename
            .replaceAll("&", "&amp;")
            .replaceAll("<", "&lt;")
            .replaceAll(">", "&gt;")
            .replaceAll('"', " & quot; ")
            .replaceAll("'", "&#039;");
        return n;
    });

    let height = window.innerHeight - 64;

    window.addEventListener("resize", (e) => {
        const newHeight = window.innerHeight - 64;
        height = newHeight;
        container.classList.remove("trans");
        table.classList.remove("trans");
        locate();
        setTimeout(() => {
            // give delay to redraw, annoying but needed
            container.classList.add("trans");
            table.classList.add("trans");
        }, 0);
    });

    const container = document.createElement("div");
    container.classList.add("container");
    container.classList.add("trans");
    document.body.appendChild(container);

    const tooltip = document.getElementById("tooltip");

    let y = 0;
    let zoom = 1.0;

    const table = document.createElement("table");
    table.classList.add("trans");
    container.appendChild(table);
    const rows = [];

    function nodeT(idx, depth, offset) {
        if (!rows[depth]) {
            rows[depth] = { el: document.createElement("tr"), offset: 0 };
        }

        if (rows[depth].offset < offset) {
            const pad = document.createElement("td");
            pad.colSpan = offset - rows[depth].offset;
            rows[depth].el.appendChild(pad);
            rows[depth].offset += offset - rows[depth].offset;
        }

        const td = document.createElement("td");
        td.innerHTML =
            "<p><span>" +
            data.nodes[idx].itemName +
            ":</span> <b>" +
            data.nodes[idx].typename +
            "</b></p>";

        td.colSpan = data.nodes[idx].size;

        td.addEventListener("mouseover", (e) => {
            const node = data.nodes[idx];
            tooltip.innerHTML =
                node.itemName +
                ": <b>" +
                node.typename +
                "</b><br/>" +
                "<ul>" +
                "<li>size = " +
                node.size +
                "</li>" +
                "<li>align = " +
                node.alignment +
                "</li>" +
                "<li>field offset = " +
                node.offset +
                "</li>" +
                "</ul>" +
                "<i>double click to focus</i>";

            tooltip.style.display = "block";
        });
        td.addEventListener("mouseleave", (_) => (tooltip.style.display = "none"));

        const totalOffset = rows[depth].offset;
        td.addEventListener("dblclick", (e) => {
            const node = data.nodes[idx];
            zoom = data.nodes[0].size / node.size;
            y = (-totalOffset / data.nodes[0].size) * zoom;
            x = 0;
            locate();
        });

        rows[depth].el.appendChild(td);
        rows[depth].offset += data.nodes[idx].size;

        if (data.nodes[idx].childrenStart !== -1) {
            for (let i = 0; i < data.nodes[idx].childrenLen; i++) {
                if (data.nodes[data.nodes[idx].childrenStart + i].size) {
                    nodeT(
                        data.nodes[idx].childrenStart + i,
                        depth + 1,
                        offset + data.nodes[data.nodes[idx].childrenStart + i].offset,
                    );
                }
            }
        }
    }

    nodeT(0, 0, 0);

    for (const row of rows) table.appendChild(row.el);

    const grid = document.createElement("div");
    grid.classList.add("grid");
    container.appendChild(grid);

    for (let i = 0; i < data.nodes[0].size / 8 + 1; i++) {
        const el = document.createElement("div");
        el.classList.add("grid-line");
        el.style.top = (i / (data.nodes[0].size / 8)) * 100 + "%";
        el.innerText = i * 8;
        grid.appendChild(el);
    }

    window.addEventListener("mousemove", (e) => {
        tooltip.style.top = e.clientY + 10 + "px";
        tooltip.style.left = e.clientX + 10 + "px";
    });

    function locate() {
        container.style.top = height * y + "px";
        container.style.height = height * zoom + "px";

        table.style.width = container.style.height;
    }

    locate();
}
