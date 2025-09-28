import {basicSetup} from "codemirror";
import {
  EditorView,
  keymap,
} from "@codemirror/view";
import {Compartment, Prec, EditorState} from "@codemirror/state";
import {emacs} from "@replit/codemirror-emacs";
import {vim} from "@replit/codemirror-vim";
import {indentWithTab} from "@codemirror/commands"
import {solarizedLight, solarizedDark} from "@uiw/codemirror-theme-solarized";
import {MergeView} from "@codemirror/merge"


// TODO(veluca): add support for typst syntax highlighting.
// Consider https://github.com/uben0/tree-sitter-typst and https://github.com/lezer-parser/import-tree-sitter as a starting point.

export class CM6Editor {
  language: Compartment = new Compartment();
  keymap: Compartment = new Compartment();
  dark: Compartment = new Compartment();
  execCallback: () => void = () => {};
  onchangeCallback: () => void = () => {};
  isReadOnly = new Compartment();
  execKeyBinding = Prec.highest(
    keymap.of([
      {
        key: "Mod-Enter",
        run: () => {
          this.execCallback();
          return true;
        },
      },
    ]),
  );
  view: EditorView;

  constructor(elementId: string) {
    const element = document.getElementById(elementId);
    if (element === null) {
      throw new Error(`"${elementId}" not found`);
    }
    this.view = new EditorView({
      extensions: [
        keymap.of([indentWithTab]),
        this.keymap.of([]),
        this.execKeyBinding,
        basicSetup,
        this.dark.of(solarizedLight),
        this.isReadOnly.of(EditorState.readOnly.of(false)),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            this.onchangeCallback();
          }
        }),
        EditorView.lineWrapping,
      ],
      parent: element,
    });
  }

  setDark(dark: boolean) {
    this.view.dispatch({
      effects: this.dark.reconfigure(dark ? solarizedDark : solarizedLight),
    });
  }

  setReadOnly(isReadonly: boolean) {
    this.view.dispatch({
      effects: this.isReadOnly.reconfigure(EditorState.readOnly.of(isReadonly)),
    });
  }

  setKeymap(keymap: string) {
    if (keymap === "vim") {
      this.view.dispatch({effects: this.keymap.reconfigure(vim())});
    } else if (keymap === "emacs") {
      this.view.dispatch({effects: this.keymap.reconfigure(emacs())});
    } else {
      this.view.dispatch({effects: this.keymap.reconfigure([])});
    }
  }

  setExec(exec: () => void) {
    this.execCallback = exec;
  }

  setOnchange(onchange: () => void) {
    this.onchangeCallback = onchange;
  }

  setText(text: string) {
    this.view.dispatch({
      changes: {
        from: 0,
        to: this.view.state.doc.length,
        insert: text,
      },
    });
  }

  getText(): string {
    return this.view.state.doc.toString();
  }
}


export function makeMergeView(id: string, first: string, second: string, dark: boolean) {
  const color = dark ? solarizedDark : solarizedLight;
  const parent = document.getElementById(id);
  if (parent === null) return;
  parent.innerHTML = '';
  new MergeView({
    a: {
      doc: first,
      extensions: [
        basicSetup,
        EditorView.editable.of(false),
        EditorState.readOnly.of(true),
        EditorView.lineWrapping,
        color
      ]
    },
    b: {
      doc: second,
      extensions: [
        basicSetup,
        EditorView.editable.of(false),
        EditorState.readOnly.of(true),
        EditorView.lineWrapping,
        color
      ]
    },
    parent
  })
}
