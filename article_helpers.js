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

init_demo_toggle_handlers();
