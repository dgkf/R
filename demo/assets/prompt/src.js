class Repl {
  #elem_container;
  #elem_input;
  #elem_highlight;
  #elem_history;

  eval;  // (input: String) => output: String;
  highlight = (input) => ["none", input];  // (input: String) => [String...]; // of style, text pairs
  validate = (elem) => true;  // (input: String) => bool;
  initial_input = "";  // String
  initial_header = "";  // String
  indent = 2;
  history = {
    "selected": undefined,  // index in log
    "log": []  // array of previous inputs
  };

  constructor(parent) {
    this.initializePromptContainer(parent)
  }

  with_eval_callback(fn) {
    this.eval = fn;
    return this
  }

  with_validate_callback(fn) {
    this.validate = fn;
    return this
  }

  with_highlight_callback(fn) {
    this.highlight = fn;
    return this
  }

  with_initial_input(input) {   
    this.initial_input = input;
    if (this.#elem_input.value.length === 0) {
      this.set(input);
      this.focus();
      this.set_cursor_pos(Infinity);
    }
    return this
  }

  with_initial_header(header) {
    this.initial_header = header;
    const node = this.#history_push("output", this.initial_header, null, false);
    node.scrollIntoView();
    return this
  }

  initializePromptContainer(parent) {
    if (parent === undefined) parent = ".prompt"
    if (typeof parent === "string") {
      document.querySelectorAll(parent).forEach((e) => {
        this.initializePromptContainer(e)
      });
      return;
    }

    // auto-append templates into document
    if (!document.querySelector("template#prompt_templates")) {
      const elem = document.createElement("template");
      elem.id = "prompt_templates";
      elem.innerHTML = this.#templates;
      document.querySelector("body").appendChild(elem);
    }

    // load html elements from templates
    const templates = prompt_templates.content;
    const history = templates.querySelector("#history").content;
    const buttons = templates.querySelector("#buttons").content;
    this.#elem_container = templates.querySelector("#container").content;
    this.#elem_input = this.#elem_container.querySelector(".prompt-input");
    this.#elem_highlight = this.#elem_container.querySelector(".prompt-highlight")
    this.#elem_history = history.querySelector(".history");

    // add history before, buttons after
    this.#elem_container.insertBefore(history, this.#elem_container.firstChild);
    this.#elem_container.appendChild(buttons);

    // add keyboard listeners
    this.#elem_input.addEventListener("keydown", (e) => {
      this.#handle_key_input(e)
    });
    this.#elem_input.addEventListener("input", (e) => {
      this.#highlight_input(e)
      this.#recalculate_rows();
    });
    this.#elem_input.addEventListener("change", (e) => { 
      this.#highlight_input(e);
    });
    // textarea.addEventListener("keyup", #handle_key_input);

    parent.appendChild(this.#elem_container);
  }

  run(code) {
    if (code === undefined) code = this.#elem_input.value;
    if (!code.trim()) { return this.clear(); }

    this.history.log.push(code);
    let input = this.#history_push("input", this.#markup_highlight(code));

    // clear prompt
    this.clear();
  
    // get result and print to history
    try { 
      let result = this.eval(code);
      let node = this.#history_push("output", result, input);
      node.scrollIntoView();
    } catch (error) {
      console.log(error);
      let node = this.#markup_unexpected_error();
      input.appendChild(node);
      node.scrollIntoView();
    }

    // restore prompt focus
    this.focus();
  };

  set(input) {
    this.#elem_input.value = input;
    this.#elem_input.rows = this.#elem_input.value.split("\n").length;
    this.#elem_input.dispatchEvent(new Event('change'));
    this.#recalculate_rows();
  };

  focus() {
    this.#elem_input.focus();  
  };

  clear() {
    const e = this.#elem_input;
    this.history.selected = null;
    e.value = '';
    e.rows = 1;
    e.dispatchEvent(new Event('change'));
    e.focus(); 
  };

  set_cursor_pos(pos) {
    if (pos > this.#elem_input.value.length) {
      pos = this.#elem_input.value.length;
    } else if (pos < 0) {
      pos = 0
    };
    this.#elem_input.selectionStart = pos;
    this.#elem_input.selectionEnd = pos;  
  };

  #recalculate_rows() {
    this.#elem_input.rows = this.#elem_input.value.split("\n").length;  
  };

  #do_history_prev() {
    const input = this.#history_increment(-1);
    this.set(input);
    event.target.selectionStart = 0;
    event.target.selectionEnd = 0;
    return input;
  };

  #do_history_next() {
    const input = this.#history_increment(1);
    this.set(input);
    this.#elem_input.selectionStart = this.#elem_input.value.length;
    this.#elem_input.selectionEnd = this.#elem_input.value.length;
    return input;
  };

  #do_smart_newline() {
    const e = this.#elem_input;
    const at = e.selectionStart;

    // split our input into lines
    const splits = e.value.slice(0, at).split("\n");
    const line = splits[splits.length - 1];

    // determine whehter our current line should introduce an indent
    const paren_open = (line.match(/[\(\{\]]/g) || []).length;
    const paren_close = (line.match(/[\)\}\]]/g) || []).length;
    const do_indent = paren_open > paren_close;

    // calculate new indentation
    const ws = line.match(/^ */g)[0].length;
    const indent = " ".repeat(ws + do_indent * this.indent);

    document.execCommand("insertText", false, '\n' + indent);
  }

  #do_smart_backspace() {
    const e = this.#elem_input;
    const at = e.selectionStart;
    const to = e.selectionEnd;
    if (at == to) {
      for (var i = 0; i < this.indent; i++) {
        if (e.value[e.selectionStart - i - 1] != ' ') break;
      }
      e.selectionStart  -= i;
    }
    document.execCommand("delete");
  }

  #do_smart_home() {
    const e = this.#elem_input;
    const at = e.selectionStart;
    var pos = e.value.lastIndexOf("\n", at - 1);
    pos = pos + e.value.slice(pos + 1).search(/[^ \n\r]/g);
    e.selectionStart = pos + 1;
    e.selectionEnd = pos + 1;  
  }

  #do_smart_block_dedent() {
    const e = this.#elem_input;
    const at = e.selectionStart;
    const to = e.selectionEnd;
    const at_nl = e.value.slice(0, at).lastIndexOf("\n");
    const slice = e.value.slice(at_nl - 1, to);

    // find newline positions and insert indent after each
    var n = 0; var row_n = 0;
    var pos = to;

    // for each line, delete an indent of spaces from start
    while (pos > at_nl) {
      pos = e.value.lastIndexOf("\n", pos - 1);
      e.selectionStart = pos + 1;
      e.selectionEnd = pos + 1;
      row_n = 0;
      for (var i = 0; i < this.indent; i ++) {
        if (e.value[e.selectionStart + i] != ' ') break;
        e.selectionEnd += 1;
        n += 1; row_n += 1;
      }
      if (row_n > 0) document.execCommand("delete");
    }

    // restore selection with new indentation
    e.selectionStart = at - row_n;
    e.selectionEnd = to - n;  
  }

  #do_smart_dedent() {
    const e = this.#elem_input;
    const at = e.selectionStart;
    e.selectionStart = e.value.lastIndexOf("\n", at) + 1;
    e.selectionEnd = e.selectionStart;
    for (var i = 0; i < this.indent; i ++) {
      if (e.value[e.selectionEnd] != ' ') break;
      e.selectionEnd += 1;
    }
    const n = e.selectionEnd - e.selectionStart;
    if (n > 0) document.execCommand("delete");
    e.selectionStart = at - n;
  }

  #do_smart_block_indent() {
    const e = this.#elem_input;
    const at = e.selectionStart;
    const to = e.selectionEnd;
    const at_nl = e.value.slice(0, at).lastIndexOf("\n");
    const slice = e.value.slice(at_nl - 1, to);

    // find newline positions and insert indent after each
    var n = 0;
    var pos = to;
    while (pos > at_nl) {
      pos = e.value.lastIndexOf("\n", pos - 1);
      e.selectionStart = pos + 1;
      e.selectionEnd = pos + 1;
      this.#do_smart_indent();
      n += 1;
    }

    // restore selection with new indentation
    e.selectionStart = at + this.indent;
    e.selectionEnd = to + n * this.indent;
  }

  #do_smart_indent() {
    document.execCommand("insertText", false, " ".repeat(this.indent));
  }

  #validate(code) {
    if (code === undefined) code = this.#elem_input.value;
    return this.validate(code);
  }

  #markup_highlight(code) {
    // Notes on performance:
    // - Currently re-highlights all input on every event... probably unnecessary
    // - Can definitely be smarter about only updating highlights when
    //   certain keys are pressed
    // - Highlighting probably only needs current line of input

    if (code === undefined) code = this.#elem_input.value;
    let stream = this.highlight(code);

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
  
    this.#elem_highlight.innerHTML = '';
    return div;
  }

  #markup_unexpected_error() {
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

    return node;
  }

  #history_push(type, content, elem, click) {
    let parent = elem || this.#elem_history;

    if (!(content instanceof Element)) {
      let node = document.createElement("div");
      let text = document.createElement("pre");
      text.textContent = content;
      node.appendChild(text);
      content = node;
    }

    content.classList.add("history-cell");
    content.classList.add(type);

    if (click === undefined) click = () => this.set(content.firstChild.innerText);
    if (click instanceof Function) content.onclick = click;
    
    parent.appendChild(content);
    return content;  
  }

  #history_clear() {
    this.#elem_history.innerHTML = '';
    return this.#history_push("output", this.initial_input);
  }

  #history_increment(n) {
    let l = this.history.log.length;
    if (!l) { return; }
    n = (n % l) + l;
    this.history.selected = ((this.history.selected || 0) + n) % l;
    return this.history.log[this.history.selected]
  }

  #highlight_input(event) {
    const div = this.#elem_highlight;
    const hl = this.#markup_highlight(this.#elem_input.value);
    div.innerHTML = '';
    div.appendChild(hl);  
  }

  #handle_key_input(event) {
    if (event.skip_prompt_handler) { return; }
    const e = this.#elem_input;

    let at_start = e.selectionStart <= 0;
    let at_end = e.selectionEnd >= e.value.length;

    // go back in history (toward end, oldest)
    if (event.key == "ArrowUp" && at_start && this.history.log.length > 0) {
      return this.#do_history_prev();

    // go forward in history (toward start, most recent)
    } else if (event.key == "ArrowDown" && at_end && this.history.log.length > 0) {
      return this.#do_history_next();

    // otherwise, if other keys are pressed, reset selected in history
    } else if (event.key != "ArrowUp" && event.key != "ArrowDown") {
      this.history.selected = null;
    }

    if (event.key == "Enter" && !event.shiftKey) {
      // if ctrl is held
      if (event.ctrlKey || event.getModifierState("Meta")) {
        this.run();
        event.preventDefault();
        return;
      }

      // if code is a complete expression, run it
      if (at_end) {
        if (this.#validate()) {
          this.run();
          event.preventDefault();
          return;
        }
      }
    } else if (event.key == "Enter") {
      this.#do_smart_newline();
      this.#recalculate_rows();
      event.preventDefault();
      return;
    }

    if (event.key == "Backspace") {
      this.#do_smart_backspace();
      this.#recalculate_rows();
      event.preventDefault();
      return;
    }

    if (event.key == "Home" || (event.key == "ArrowLeft" && event.metaKey)) {
      this.#do_smart_home();
      event.preventDefault();
      return;
    }

    // Tab - indent groups, or insert a indent worth of spaces
    if (event.key == "Tab") {
      const is_block = e.selectionStart != e.selectionEnd;
      if (event.shiftKey && is_block) this.#do_smart_block_dedent(); 
      else if (event.shiftKey) this.#do_smart_dedent();
      else if (!event.shiftKey && is_block) this.#do_smart_block_indent();
      else this.#do_smart_indent();

      event.preventDefault();
      return;
    }
  };

  #templates = `
    <template id="container">
      <div class="prompt-input-container">
        <textarea class="prompt-input" name="prompt" rows="1" spellcheck="true" autocomplete="off" autocapitalize="none"></textarea>
        <div class="prompt-highlight"></div>
      </div>
    </template>

    <template id="history">
      <div class="history-scroll">
        <div class="history-scroll-pad"></div>
        <div class="history"></div>
      </div>
    </template>

    <template id="buttons">
      <div class="flex-row">
        <div class="btn clear">clear</div>
        <div class = "btn-group">
          <div id="light-mode" class="icon i-sun" onclick="theme_light()" alt="switch to light mode"></div>
          <div id="dark-mode" class="icon i-moon" onclick="theme_dark()" alt="switch to dark mode" style="display: none;"></div>
        </div>
        <div class="btn-group">
          <a class="btn icon i-github" href="https://github.com/dgkf/R" target="_blank" alt="GitHub"></a>
        </div>
        <div class="btn-group">
          <div class="btn btn-slim icon i-down-arrow" onclick="font_size_adjust(8/9)" alt="decrease font size"></div>
          <div class="btn btn-slim font-reset" onclick="font_size(1)">Aa</div>
          <div class="btn btn-slim icon i-up-arrow" onclick="font_size_adjust(9/8)" alt="increase font size"></div>
        </div>
        <div class="btn-group dropup">
          <div class="btn icon i-translate" alt="localization"></div>
          <div class="dropup-content">
            <p><a href="?locale=en">English</a></p>
            <p><a href="?locale=es">Español</a></p>
            <p><a href="?locale=cn">中文</a></p>
          </div>
        </div>
        <div class="btn submit" >run</div>
      </div>
    </template>
  `
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


