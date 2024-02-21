addEventListener("DOMContentLoaded", (event) => {
  theme_dark();
  let prompt = document.getElementById("prompt");
  prompt.addEventListener("keydown", prompt_input);
  prompt_resize();
  prompt.focus();
  prompt_cursor(Infinity);
});

function prompt_input(event) {
  let at = event.target.selectionStart;
  let to = event.target.selectionEnd;

  let at_start = at <= 0;
  let at_end = to >= event.target.value.length;

  // go back in history (toward end, oldest)
  if (event.key == "ArrowUp" && at_start) {
    return prompt_set(history_increment(-1));

  // go forward in history (toward start, most recent)
  } else if (event.key == "ArrowDown" && at_end) {
    return prompt_set(history_increment(1));

  // otherwise, if other keys are pressed, reset selected in history
  } else if (event.key != "ArrowUp" && event.key != "ArrowDown") {
    r.history.selected = null;
  }

  // possibly submit code
  if (event.key == "Enter") {
    // if ctrl is held
    if (event.ctrlKey || event.getModifierState("Meta")) {
      run();
      event.preventDefault();
      return;
    }

    // if code is a complete expression, run it
    if (at_end) {
      if (r.validate(prompt().value)) {
        run();
        event.preventDefault();
        return;
      }
    }
  }

  // add or remove lines based on input
  if (event.key == "Enter") {
    let prompt = document.getElementById("prompt");
    prompt.rows = prompt.value.split("\n").length + 1;
  }

  if (event.key == "Backspace") {
    let prompt = document.getElementById("prompt");
    let val = prompt.value;
    console.log(at, to);
    console.log("'" + prompt.value.substring(at, to + 1) + "'");
    prompt.rows = prompt.value.split("\n").length - (prompt.value.substring(at - 1, to).split("\n").length - 1)
  }
}

function highlight(stream) {
  var div = document.createElement("div");
  var pre = document.createElement("pre");
  div.appendChild(pre);

  var i = 0;
  var j = 0;
  while (i < stream.length) {
    let style = stream[i];
    let texts = stream[i+1].split("\n");

    j = 0;
    while (j < texts.length) {
      if (j > 0) pre.appendChild(document.createElement("br"));
      var span = document.createElement("span");
      span.className = "style-" + style;
      span.innerHTML = texts[j];
      pre.appendChild(span);
      j += 1;
    }

    i += 2;
  }
  return div
}

function history_push(type, content, elem) {
  let parent = elem || document.getElementById("history");

  if (!(content instanceof Element)) {
    let node = document.createElement("div");
    let text = document.createElement("pre");
    text.textContent = content;
    node.appendChild(text);
    content = node;
  }

  content.className += " history-cell " + type;
  content.onclick = () => prompt_set(content.firstChild.innerText);
  parent.appendChild(content);
  return content;
}

function history_increment(n) {
  let l = r.history.log.length;
  if (!l) { return; }
  n = (n % l) + l;
  r.history.selected = ((r.history.selected || 0) + n) % l;
  return r.history.log[r.history.selected]
}

function unexpected_error(elem) {
  let parent = elem || document.getElementById("history")
  let node = document.createElement("div");
  node.className = "history-cell error";

  var text = document.createElement("pre");
  text.textContent = "Error: An unexpected error was encountered!";
  node.appendChild(text);

  var text = document.createElement("span");
  text.textContent = "Why not ";
  node.appendChild(text);

  var link = document.createElement("a");
  link.href = "https://github.com/dgkf/R/issues";
  link.textContent = "submit an issue";
  node.appendChild(link);
  
  var text = document.createElement("span");
  text.textContent = "?";
  node.appendChild(text);

  parent.appendChild(node);
  return node;
}

function run() {
  let prompt = document.getElementById("prompt");

  // read code and print to history
  let code = prompt.value;
  if (!code.trim()) { return prompt_clear(); }

  r.history.log.push(code);
  let input = history_push("input", highlight(r.highlight(code)));
  
  // get result and print to history
  try { 
    let result = r.eval(code);
    let node = history_push("output", result, input);
    node.scrollIntoView();
  } catch (error) {
    console.log(error);
    let node = unexpected_error(input);
    node.scrollIntoView();
  }

  // clear prompt & restore focus
  prompt_clear();
}

function prompt() {
  return document.getElementById("prompt")
}

function prompt_clear() {
  r.history.log.selected = null;
  let elem = prompt();
  elem.value = '';
  elem.rows = 1;
  elem.focus();
}

function prompt_set(input) {
  let elem = prompt();
  elem.value = input;
  prompt_resize();
}

function prompt_cursor(n) {
  let elem = prompt();
  if (n > elem.value.length) n = elem.value.length;
  if (n < 0) n = 0;
  elem.selectionStart = n;
}

function prompt_resize() {
  let elem = prompt()
  elem.rows = elem.value.split("\n").length;       
}

function clear_history() {
  document.getElementById("history").innerHTML = '';
  history_push("output", r.header);
}

function font_size(value) {
  let root = document.querySelector(":root");
  root.style.setProperty("--font-scale", value);
}

function font_size_adjust(perc) {
  let root = document.querySelector(":root");
  let rootstyle = getComputedStyle(root);
  let scale = rootstyle.getPropertyValue("--font-scale");
  font_size(scale * perc)
}

function theme_light_icon() {
  return document.getElementById("light-mode");
}

function theme_dark_icon() {
  return document.getElementById("dark-mode");
}

function theme_light() {
  let root = document.querySelector(":root");
  root.style.setProperty("--bg", "var(--light-bg)")
  root.style.setProperty("--fg", "var(--light-fg)")

  let body = document.querySelector("body");
  body.classList.add("light");
  body.classList.remove("dark");

  theme_light_icon().style.display = "none";
  theme_dark_icon().style.display = "initial";
}

function theme_dark() {
  let root = document.querySelector(":root");
  root.style.setProperty("--bg", "var(--dark-bg)")
  root.style.setProperty("--fg", "var(--dark-fg)")

  let body = document.querySelector("body");
  body.classList.add("dark");
  body.classList.remove("light");

  theme_light_icon().style.display = "initial";
  theme_dark_icon().style.display = "none";
}
