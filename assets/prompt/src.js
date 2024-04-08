class Repl {
  #elem_container;
  #elem_input;
  #elem_highlight;
  #elem_diagnostics;
  #elem_output;
  #timeouts = {};

  eval;  // (input: String) => output: String;
  highlight = (input) => ["none", input];  // (input: String) => [String...]; // of style, text pairs
  validate = (elem) => true;  // (input: String) => bool;
  initial_input = "";         // String
  initial_header = "";        // String
  initial_run = false;        // bool: whether to immediately run input
  indent = 2;

  output = {
    "mode": "single",         // "history" or "single"
    "location": "above",      // "above" or "below"
    "selected": undefined,    // index in log
    "log": []                 // array of previous inputs
  };

  constructor(parent, params) {
    if (parent === undefined) parent = ".prompt";
    if (typeof parent === "string") {
      document.querySelectorAll(parent).forEach((e) => {
        new this.constructor(e, params)
      });
      return;
    }

    if (params === undefined) params = {};
    const data = parent.dataset;

    this.eval = params.eval ||
      this.eval;
    this.validate = params.validate ||
      this.validate;
    this.highlight = params.highlight ||
      this.highlight;
    this.indent = params.indent ||
      data.indent ||
      this.indent;
    this.output.mode = (params.output && params.output.mode) ||
      data.outputMode ||
      this.output.mode;
    this.output.location = (params.output && params.output.location ) || 
      data.outputLocation || 
      this.output.location;

    this.initialize_prompt_container(parent)

    this.initial_input = params.initial_input ||
      (data.initialInput && data.initialInput.replace(/\\n/g, "\n")) ||
      this.initial_input;
    if (this.initial_input) this.with_initial_input(this.initial_input);

    this.initial_header = params.initial_header ||
      data.initialHeader ||
      this.initial_header;
    if (this.initial_header) this.with_initial_header(this.initial_header);

    this.initial_run = params.initial_run ||
      data.initialRun ||
      this.initial_run;

    if (this.initial_run) this.run();
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
    const node = this.#output_push("output", this.initial_header, null, false);
    node.scrollIntoView();
    return this
  }

  initialize_prompt_container(parent) {
    // auto-append templates into document
    if (!document.querySelector("template#prompt_templates")) {
      const elem = document.createElement("template");
      elem.id = "prompt_templates";
      elem.innerHTML = this.#templates;
      document.querySelector("body").appendChild(elem);
    }

    // load html elements from templates
    var templates = prompt_templates.content;
    var output = templates.querySelector("#output").content.cloneNode(true);
    this.#elem_container = templates.querySelector("#container").content.cloneNode(true);
    this.#elem_input = this.#elem_container.querySelector(".prompt-input");
    this.#elem_highlight = this.#elem_container.querySelector(".prompt-highlight")
    this.#elem_diagnostics = this.#elem_container.querySelector(".prompt-diagnostics")
    this.#elem_output = output.querySelector(".output-container");

    // add output before, buttons after
    if (this.output.location === "above") {
      this.#elem_container.insertBefore(output, this.#elem_container.firstChild);
    } else if (this.output.location === "below") {
      this.#elem_container.appendChild(output);
    }

    // update hotkey indicators by OS
    var run = this.#elem_container.querySelector(".prompt-run");
    if (window.navigator.platform.startsWith("Mac")) {
      run.title = "Cmd + Enter"
    } else {
      run.title = "Ctrl + Enter"
    }

    // add run button 
    run.addEventListener("click", (e) => this.run());

    // add keyboard listeners
    this.#elem_input.addEventListener("keydown", (e) => {
      this.#handle_key_input(e)
    });
    this.#elem_input.addEventListener("input", (e) => {
      this.#highlight_input(e)
      this.#validate()
      this.#recalculate_rows();
    });
    this.#elem_input.addEventListener("change", (e) => { 
      // this.#highlight_input(e);
      // this.#validate()
    });

    parent.appendChild(this.#elem_container);
  }

  run(code) {
    if (code === undefined) code = this.#elem_input.value;
    if (!code.trim()) { return this.clear(); }

    this.output.log.push(code);

    var history_elem = this.#elem_output;
    if (this.output.mode === "history") {
      history_elem = this.#output_push("input", this.#markup_highlight(code));
      this.clear();
    }
  
    // get result and print to output
    try { 
      let output_elem = this.#output_push("output", "", history_elem, false);
      let result = this.eval(code, (x) => output_elem.innerText += x);
      if (this.output.mode === "single") this.#elem_output.innerHTML = "";
      output_elem.innerText += result;
      output_elem.scrollIntoView();
    } catch (error) {
      console.log(error);
      let node = this.#markup_unexpected_error();
      output.appendChild(node);
      node.scrollIntoView();
    }

    // restore prompt focus
    this.focus();
  };

  set(input) {
    this.#elem_input.value = input;
    this.#elem_input.rows = this.#elem_input.value.split("\n").length;
    this.#elem_input.dispatchEvent(new Event('input'));
    this.#recalculate_rows();
  };

  focus() {
    this.#elem_input.focus();  
  };

  clear() {
    const e = this.#elem_input;
    this.output.selected = null;
    e.value = '';
    e.rows = 1;
    e.dispatchEvent(new Event('input'));
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

  #do_output_prev() {
    const input = this.#output_increment(-1);
    this.set(input);
    event.target.selectionStart = 0;
    event.target.selectionEnd = 0;
    return input;
  };

  #do_output_next() {
    const input = this.#output_increment(1);
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
    let errors = this.validate(code);

    // console.log(errors);
    // for (i in errors) {
    //   console.log(i);
    //   x = { "start": e[i].start(), "end": e[i].end(), "message": e[i].message() };
    //   console.log(x)
    // }

    if (errors.length > 0) {
      clearTimeout(this.#timeouts.diagnostics);
      this.#timeouts.diagnostics = setTimeout(() => {
        const markup = this.#markup_errors(code, errors);
        this.#timeouts.diagnostics && this.#elem_diagnostics.replaceChildren(markup);
      }, 1000)
    } else {
      clearTimeout(this.#timeouts.diagnostics);
      this.#elem_diagnostics.replaceChildren([]);
    }   

    return this.validate(code).length === 0;
  }

  #markup_errors(code, errors) {
    if (code === undefined) code = this.#elem_input.value;

    var frag = document.createDocumentFragment();
    errors.forEach((e) => {
      var div = document.createElement("div");

      var pad = document.createElement("span");
      pad.textContent = code.slice(0, e.start() - 1).replace(/[^\n]/g, " ");
      div.appendChild(pad);

      var err = document.createElement("span");
      err.textContent = " ".repeat(e.end() - e.start() + 1);
      err.classList.add("style-error");
      err.title = e.message();
      div.appendChild(err)

      frag.appendChild(div);            
    })
    
    return frag;    
  }

  #markup_highlight(code) {
    // Notes on performance:
    // - Currently re-highlights all input on every event... probably unnecessary
    // - Can definitely be smarter about only updating highlights when
    //   certain keys are pressed
    // - Highlighting probably only needs current line of input

    if (code === undefined) code = this.#elem_input.value;
    let stream = this.highlight(code);
    const frag = document.createDocumentFragment();

    var i = 0;
    var j = 0;
    while (i < stream.length) {
      let style = stream[i];
      var span = document.createElement("span");
      span.className = "style-" + style;
      span.textContent = stream[i+1];
      frag.appendChild(span);
      i += 2;
    }

    return frag;
  }

  #markup_unexpected_error() {
    let node = document.createElement("div");
    node.className = "output-cell error";

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

  #output_push(type, content, elem, click) {
    let parent = elem || this.#elem_output;

    if (typeof content === "string") {
      let node = document.createElement("div");
      let text = document.createElement("pre");
      text.textContent = content;
      node.appendChild(text);
      content = node;
    } else if (content instanceof DocumentFragment) {
      let node = document.createElement("div");
      node.replaceChildren(content);
      content = node;
    }

    let text_content = content.textContent;

    content.classList.add("output-cell");
    content.classList.add(type);

    if (click === undefined) click = (event) => {
      // ignore in case where a text selection was made
      if (document.getSelection().type == "Range") { return };
      this.set(text_content);
      this.focus();
      this.set_cursor_pos(Infinity);
    };

    if (click instanceof Function) {
      content.onclick = click;

      const share = document.createElement("div")
      share.classList.add("output-share")
      share.onclick = () => {
        const loc = window.location;
        var params = new URLSearchParams(loc.search);
        params.set("expr", btoa(unescape(encodeURIComponent(text_content))).replace(/=*$/, ""));
        const suffix = loc.pathname + "?" + params.toString();
        const url = loc.origin + suffix;
        window.history.replaceState("object or string", "", suffix);
        navigator.clipboard.writeText(url);
      }

      content.appendChild(share);
    }
    
    parent.appendChild(content);
    return content;  
  }

  #output_clear() {
    this.#elem_output.innerHTML = '';
    return this.#output_push("output", this.initial_header);
  }

  #output_increment(n) {
    let l = this.output.log.length;
    if (!l) { return; }
    n = (n % l) + l;
    this.output.selected = ((this.output.selected || 0) + n) % l;
    return this.output.log[this.output.selected]
  }

  #highlight_input(event) {
    const div = this.#elem_highlight;
    const hl = this.#markup_highlight(this.#elem_input.value);
    div.replaceChildren(hl);
  }

  #handle_key_input(event) {
    if (event.skip_prompt_handler) { return; }
    const e = this.#elem_input;

    let at_start = e.selectionStart <= 0;
    let at_end = e.selectionEnd >= e.value.length;

    // go back in output (toward end, oldest)
    if (event.key == "ArrowUp" && at_start && this.output.log.length > 0) {
      return this.#do_output_prev();

    // go forward in output (toward start, most recent)
    } else if (event.key == "ArrowDown" && at_end && this.output.log.length > 0) {
      return this.#do_output_next();

    // otherwise, if other keys are pressed, reset selected in output
    } else if (event.key != "ArrowUp" && event.key != "ArrowDown") {
      this.output.selected = null;
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
        <textarea class="prompt-input" name="prompt" rows="1" spellcheck="false" autocomplete="off" autocapitalize="none"></textarea>
        <div class="prompt-highlight"></div>
        <div class="prompt-diagnostics"></div>
        <div class="prompt-run" title="Meta + Enter"></div>
      </div>
    </template>

    <template id="output">
      <div class="output-scroll">
        <div class="output-scroll-pad"></div>
        <div class="output-container"></div>
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

function theme_toggle() {
  const root = document.documentElement;
  if (root.classList.contains("light")) {
    root.classList.remove("light");
    root.classList.add("dark");
  } else if (root.classList.contains("dark")) {
    root.classList.remove("dark");
    root.classList.add("light");
  } else if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    root.classList.add("dark");
    theme_toggle();
  } else {
    root.classList.add("light");
    theme_toggle();
  }
}

