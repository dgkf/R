function prompt() {
  return document.getElementById("prompt-input");
}

function initialize_prompt() {
  theme_dark();

  let prompt = document.getElementById("prompt-input");
  prompt.addEventListener("keydown", (event) => prompt_input(prompt, event));
  prompt.addEventListener("input", (event) => prompt_highlight(prompt, event));
  // prompt.addEventListener("keyup", (event) => prompt_highlight(prompt, event));
  prompt.addEventListener("change", (event) => prompt_highlight(prompt, event));

  node = clear_history();
  node.scrollIntoView();

  prompt_set(r.prompt);
  prompt_focus();
  prompt_cursor(Infinity);
};

function prompt_highlight(prompt, event) {
  // Notes on performance:
  // - Currently re-highlights all input on every event... probably unnecessary
  // - Can definitely be smarter about only updating highlights when
  //   certain keys are pressed
  // - Highlighting probably only needs current line of input
  const hl_div = document.getElementById("prompt-display");
  const hl = highlight(r.highlight(prompt.value));
  hl_div.innerHTML = '';
  hl_div.appendChild(hl);  
}

function prompt_input(prompt, event) {
  if (event.skip_prompt_handler) { return; }

  const indent_chars = 2;
  let at = event.target.selectionStart;
  let to = event.target.selectionEnd;

  let at_start = at <= 0;
  let at_end = to >= event.target.value.length;

  // go back in history (toward end, oldest)
  if (event.key == "ArrowUp" && at_start && r.history.log.length > 0) {
    prompt_set(history_increment(-1));
    event.target.selectionStart = 0;
    event.target.selectionEnd = 0;
    return; 

  // go forward in history (toward start, most recent)
  } else if (event.key == "ArrowDown" && at_end && r.history.log.length > 0) {
    prompt_set(history_increment(1));
    event.target.selectionStart = event.target.value.length;
    event.target.selectionEnd = event.target.value.length;
    return;

  // otherwise, if other keys are pressed, reset selected in history
  } else if (event.key != "ArrowUp" && event.key != "ArrowDown") {
    r.history.selected = null;
  }

  // possibly submit code
  if (event.key == "Enter" && !event.shiftKey) {
    // if ctrl is held
    if (event.ctrlKey || event.getModifierState("Meta")) {
      run();
      event.preventDefault();
      return;
    }

    // if code is a complete expression, run it
    if (at_end) {
      if (r.validate(prompt.value)) {
        run();
        event.preventDefault();
        return;
      }
    }
  }

  // add or remove lines based on input
  if (event.key == "Enter") {
    // split our input into lines
    const splits = prompt.value.slice(0, at).split("\n");
    const line = splits[splits.length - 1];

    // determine whehter our current line should introduce an indent
    const paren_open = (line.match(/[\(\{\]]/g) || []).length;
    const paren_close = (line.match(/[\)\}\]]/g) || []).length;
    const do_indent = paren_open > paren_close;

    // calculate new indentation
    const ws = line.match(/^ */g)[0].length;
    const indent = " ".repeat(ws + do_indent * indent_chars);

    event.preventDefault();
    document.execCommand("insertText", false, '\n' + indent);
    prompt_resize();
    return;
  }

  // Backspace - delete indent groups of spaces
  if (event.key == "Backspace") {
    if (at == to) {
      for (var i = 0; i < indent_chars; i++) {
        if (prompt.value[event.target.selectionStart - i - 1] != ' ') break;
      }
      event.target.selectionStart -= i;
    }
    document.execCommand("delete");
    event.preventDefault();
    prompt_resize();
    return;
  }

  // Home - goes to start of line
  if (event.key == "Home" || (event.key == "ArrowLeft" && event.metaKey)) {
    pos = prompt.value.lastIndexOf("\n", at - 1);
    pos = pos + prompt.value.slice(pos + 1).search(/[^ \n\r]/g);
    event.target.selectionStart = pos + 1;
    event.target.selectionEnd = pos + 1;
    event.preventDefault();
    return;
  }

  // Tab - indent groups, or insert a indent worth of spaces
  if (event.key == "Tab") {
    const at_nl = prompt.value.slice(0, at).lastIndexOf("\n");
    const slice = prompt.value.slice(at_nl - 1, to);

    if (event.shiftKey && at != to) {
      // find newline positions and insert indent after each
      var n = 0; var row_n = 0;
      var pos = to;

      // for each line, delete an indent of spaces from start
      while (pos > at_nl) {
        pos = prompt.value.lastIndexOf("\n", pos - 1);
        event.target.selectionStart = pos + 1;
        event.target.selectionEnd = pos + 1;
        row_n = 0;
        for (var i = 0; i < indent_chars; i ++) {
          if (prompt.value[event.target.selectionStart + i] != ' ') break;
          event.target.selectionEnd += 1;
          n += 1; row_n += 1;
        }
        if (row_n > 0) document.execCommand("delete");
      }

      // restore selection with new indentation
      event.target.selectionStart = at - row_n;
      event.target.selectionEnd = to - n;

    } else if (event.shiftKey) {
      event.target.selectionStart = prompt.value.lastIndexOf("\n", at) + 1;
      event.target.selectionEnd = event.target.selectionStart;
      for (var i = 0; i < indent_chars; i ++) {
        if (prompt.value[event.target.selectionEnd] != ' ') break;
        event.target.selectionEnd += 1;
      }
      const n = event.target.selectionEnd - event.target.selectionStart;
      if (n > 0) document.execCommand("delete");
      event.target.selectionStart = at - n;

    } else if (!event.shiftKey && at != to){
      // find newline positions and insert indent after each
      var n = 0;
      var pos = to;
      while (pos > at_nl) {
        pos = prompt.value.lastIndexOf("\n", pos - 1);
        event.target.selectionStart = pos + 1;
        event.target.selectionEnd = pos + 1;
        document.execCommand("insertText", false, " ".repeat(indent_chars));
        n += 1;
      }

      // restore selection with new indentation
      event.target.selectionStart = at + indent_chars;
      event.target.selectionEnd = to + n * indent_chars;
    } else {
      document.execCommand("insertText", false, " ".repeat(indent_chars));
    }

    prompt.dispatchEvent(new Event('change'));
    event.preventDefault();
    return;
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

function history_push(type, content, elem, log) {
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
  // read code and print to history
  let code = prompt().value;
  if (!code.trim()) { return prompt_clear(); }

  r.history.log.push(code);
  let input = history_push("input", highlight(r.highlight(code)));

  // clear prompt
  prompt_clear();
  
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

  // restore prompt focus
  prompt_focus();
}

function prompt_clear() {
  r.history.selected = null;
  let elem = prompt();
  elem.value = '';
  elem.rows = 1;
  elem.dispatchEvent(new Event('change'));
  elem.focus();
}

function prompt_focus()  {
  let elem = prompt();
  elem.focus();  
}

function prompt_set(input) {
  let elem = prompt();
  elem.value = input;
  elem.rows = elem.value.split("\n").length;
  elem.dispatchEvent(new Event('change'));
  prompt_resize();
}

function prompt_cursor(n) {
  let elem = prompt();
  if (n > elem.value.length) n = elem.value.length;
  if (n < 0) n = 0;
  elem.selectionStart = n;
  elem.selectionEnd = n;
}

function prompt_resize() {
  let elem = prompt()
  elem.rows = elem.value.split("\n").length;       
}

function clear_history() {
  document.getElementById("history").innerHTML = '';
  return history_push("output", r.header);
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
