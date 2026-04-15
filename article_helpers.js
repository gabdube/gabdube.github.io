const demo_state_key = `${document.body.dataset.sessionKey}_demo_state`
let toggle_demo_state = sessionStorage.getItem(demo_state_key) || "both";
const min_width_for_both = 600;

function updateBodyClasses() {
    const classes = document.body.classList;
    classes.remove("focus-article");
    classes.remove("focus-demo");
    if (toggle_demo_state === "article") {
        classes.add("focus-article");
    } else if (toggle_demo_state === "demo") {
        classes.add("focus-demo");
    }

    sessionStorage.setItem(demo_state_key, toggle_demo_state);
}

async function toggleDemo() {
    if (toggle_demo_state === "both") {
        toggle_demo_state = "article";
    } else if (toggle_demo_state === "article") {
        toggle_demo_state = "demo";
    } else if (toggle_demo_state === "demo") {
        if (document.body.offsetWidth < min_width_for_both) {
            toggle_demo_state = "article";
        } else {
            toggle_demo_state = "both";
        }
    }

    updateBodyClasses();
}

function init_demo_toggle_handlers() {
    const body = document.body;
    if (body.offsetWidth < min_width_for_both) {
        toggle_demo_state = "article";
    }

    document.getElementById("toggleDemo")?.addEventListener("click", () => {
        toggleDemo();
    });

    window.addEventListener("resize", () => {
        if (body.offsetWidth < min_width_for_both && toggle_demo_state === "both") {
            toggle_demo_state = "article";
            updateBodyClasses();
        }
    });

    document.getElementById("resetDemo")?.addEventListener("click", () => {
        if (window.demo) {
            window.demo.reload = true;
        }
    });

    updateBodyClasses();
}

// CodeView functionality
function file_name(path) {
    return path.split(/[\\/]/).pop() || "";
}

const CODE_VIEW_DATA = {
    tabs: [],
    current_tab: -1,
};

async function toggle_codeview() {
    const demo = document.getElementById("demo");
    const classes = demo.classList;

    if (!classes.contains("show-tabs")) {
        classes.add("show-tabs");
    }

    if (classes.contains("show-demo")) {
        classes.remove("show-demo");
        classes.add("show-code");
    }
}

async function toggle_demo_codeview() {
    const demo = document.getElementById("demo");
    const classes = demo.classList;
    if (classes.contains("show-code")) {
        classes.remove("show-code");
        classes.add("show-demo");
    }
}

function fetch_cache(target_url) {
    for (const tab of CODE_VIEW_DATA.tabs) {
        if (tab.url === target_url) {
            return tab;
        }
    }

    return null;
}

function refresh_tab_display() {
    const header = document.getElementById("demoheader");
    if (!header) { return; }

    while (header.children.length > 1) {
        header.removeChild(header.children[1]);
    }

    for (const [index, tab] of CODE_VIEW_DATA.tabs.entries()) {
        if (index == CODE_VIEW_DATA.current_tab) {
            tab.tab_element.classList.add("active");
        } else {
            tab.tab_element.classList.remove("active");
        }
        header.appendChild(tab.tab_element);
    }

    const show_demo = document.getElementById("demoheaderShowDemo");
    if (CODE_VIEW_DATA.current_tab === -1) {
        show_demo.classList.add("active");
    } else {
        show_demo.classList.remove("active");
    }
}

function show_tab(tab) {
    const codeview = document.getElementById("codeview");
    codeview.innerHTML = "";
    codeview.appendChild(tab.content);
    CODE_VIEW_DATA.current_tab = tab.index;
    codeview.scrollTo({left: tab.scrollx, top: tab.scrolly});
    toggle_codeview();
}

function create_cross_svg() {
    const CROSS_SVG =
`
<svg fill="rgb(27, 27, 27)" height="12px" width="12px" version="1.1" id="Capa_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
	 viewBox="0 0 460.775 460.775" xml:space="preserve">
<path d="M285.08,230.397L456.218,59.27c6.076-6.077,6.076-15.911,0-21.986L423.511,4.565c-2.913-2.911-6.866-4.55-10.992-4.55
	c-4.127,0-8.08,1.639-10.993,4.55l-171.138,171.14L59.25,4.565c-2.913-2.911-6.866-4.55-10.993-4.55
	c-4.126,0-8.08,1.639-10.992,4.55L4.558,37.284c-6.077,6.075-6.077,15.909,0,21.986l171.138,171.128L4.575,401.505
	c-6.074,6.077-6.074,15.911,0,21.986l32.709,32.719c2.911,2.911,6.865,4.55,10.992,4.55c4.127,0,8.08-1.639,10.994-4.55
	l171.117-171.12l171.118,171.12c2.913,2.911,6.866,4.55,10.993,4.55c4.128,0,8.081-1.639,10.992-4.55l32.709-32.719
	c6.074-6.075,6.074-15.909,0-21.986L285.08,230.397z"/>
</svg>
`;

    const parser = new DOMParser();
    const svgDoc = parser.parseFromString(CROSS_SVG, "image/svg+xml");
    const svgElement = svgDoc.documentElement;
    return svgElement;
}

async function add_tab(url) {
    async function generate_code_element(url) {
        let code_string = await (await fetch(url)).text();
        code_string = code_string.replaceAll("<", "&lt;")
        code_string = code_string.replaceAll(">", "&gt;")

        const code_elem = document.createElement("code");
        code_elem.innerHTML = code_string;
        hljs.highlightElement(code_elem);

        const content = document.createElement("pre");
        content.appendChild(code_elem);

        return content;
    }

    function generate_tab_element(name) {
        const range = document.createRange();
        const fragment = range.createContextualFragment(`<div class="demoheader-item"><span>${name}</span></div>`);
        return fragment.firstChild;
    }

    function select_tab(tab) {
        if (CODE_VIEW_DATA.current_tab !== tab.index) {
            CODE_VIEW_DATA.current_tab = tab.index;
            show_tab(tab);
            refresh_tab_display();
        }
    }

    async function close_tab(tab) {
        if (CODE_VIEW_DATA.current_tab == tab.index) {
            CODE_VIEW_DATA.current_tab -= 1;
            if (CODE_VIEW_DATA.current_tab >= 0) {
                show_tab(CODE_VIEW_DATA.tabs[CODE_VIEW_DATA.current_tab]);
            } else {
                toggle_demo_codeview();
            }
        } else if (CODE_VIEW_DATA.current_tab > tab.index) {
            CODE_VIEW_DATA.current_tab -= 1;
        }

        CODE_VIEW_DATA.tabs.splice(tab.index, 1);

        for (const [index, tab] of CODE_VIEW_DATA.tabs.entries()) {
            tab.index = index;
        }

        refresh_tab_display();
    }

    const index = CODE_VIEW_DATA.tabs.length;
    const name = file_name(url);
    const tab_element = generate_tab_element(name);
    const cross_svg = create_cross_svg();
    const content = await generate_code_element(url);

    const tab_data = {
        index,
        scrolly: 0,
        scrollx: 0,
        name,
        url,
        tab_element,
        content,
    };

    tab_element.title = url;
    tab_element.appendChild(cross_svg);
    tab_element.addEventListener("click", () => select_tab(tab_data));
    cross_svg.addEventListener("click", (event) => { close_tab(tab_data); event.stopPropagation(); });

    CODE_VIEW_DATA.tabs.push(tab_data);
    CODE_VIEW_DATA.current_tab = index;

    return tab_data;
}

function scroll_to_code(code) {
    const codeview = document.getElementById("codeview");
    const code_element = codeview.firstChild?.firstChild;
    if (!code_element) {
        console.error("Failed to find code element in codeview");
        return;
    }

    if (code.includes('\n') || code.includes('\r')) {
        console.error("scroll to node doesn't support newlines")
        return;
    };

    const children = code_element.childNodes;
    let found = null;
    let line_text = "";

    let i = 0;
    while (!found) {
        const child_item = children.item(i);
        if (!child_item) {
            break;
        }

        let text = child_item.textContent || "";
        let split_newlines = text.split("\n");
        if (split_newlines.length === 1) {
            i += 1;
            line_text += split_newlines[0];
            continue;
        }

        for (let j = 0; j < split_newlines.length - 1; j++) {
            line_text += split_newlines[j];
            if (line_text.length > 0) {
                const code_escaped = code.replaceAll("(", "\\(").replaceAll(")", "\\)");
                if ((line_text.match(code_escaped)?.length || 0) > 0) {
                    found = child_item;
                    break;
                }
                line_text = "";
            }
        }

        line_text = split_newlines[split_newlines.length - 1];
        i += 1;
    }

    // Hacky but good enough
    if (found) {
        let previous = found.previousSibling;
        if (!previous) { return; }

        while (!previous.scrollIntoView) {
            previous = previous.previousSibling;
        }

        previous.scrollIntoView({behavior: "smooth"});
    }
}

async function load_code(link) {
    const url = link.dataset.url;
    if (!url) {
        console.error("No url defined in code link", link);
        return;
    }

    let tab = fetch_cache(url);
    if (!tab) {
        tab = await add_tab(url);
    }

    show_tab(tab);

    if (link.dataset.goto) {
        scroll_to_code(link.dataset.goto)
    }

    refresh_tab_display();
}

/// hljs is very slow: this function is 12 time more expensive to call than initializing the entire game demo, so we wrap it inside a timeout
/// to not cause a delay at page load time. I should replace hljs with a rust/wasm implementation.
function init_codeview() {
    const TIMEOUT = 500;
    setTimeout(() => {
        const content = document.getElementById("content");
        const demo = document.getElementById("demo");
        const codeview = document.getElementById("codeview");
        if (!content || !demo || !codeview ) {
            return;
        }

        for (const link of content.getElementsByClassName("code-link")) {
            link.innerHTML = `${link.innerHTML}<span class="code-link-icon"></span>`;

            link.addEventListener("click", () => {
                if (demo.offsetWidth === 0) {
                    window.open(link.dataset.url2, "blank");
                } else {
                    load_code(link);
                    toggle_codeview();
                }
            });
        }

        for (const code of document.querySelectorAll('pre code')) {
            hljs.highlightElement(code);
        }

        const showdemo = document.getElementById("demoheaderShowDemo");
        if (showdemo) {
            showdemo.addEventListener("click", async () => {
                CODE_VIEW_DATA.current_tab = -1;
                toggle_demo_codeview();
                refresh_tab_display();
            });
        }

        codeview.addEventListener("scrollend", () => {
            const tab = CODE_VIEW_DATA.tabs[CODE_VIEW_DATA.current_tab];
            tab.scrolly = codeview.scrollTop;
            tab.scrollx = codeview.scrollLeft;
        });
    }, TIMEOUT);
}

init_demo_toggle_handlers();
init_codeview();