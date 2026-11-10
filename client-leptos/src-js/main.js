import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorThemeExtensions } from "./style.js";
//import "./css/style.css";
//import "./css/themes.css";

/* npm run build */

//console.log("editor-module");

//import { marked } from "marked";
//marked.setOptions({ gfm: true, breaks: true });
//

/** @type HTMLDivElement */
let wrapper;
/** @type EditorView */
let editor;

/**
 * @param {string}  content
 */
export function editor_start(content) {
  editor_close();
  setTimeout(() => {
    editor = new EditorView({
      state: state(content),
      parent: document.querySelector("#editor"),
    });
  }, 50);
}

export function editor_close() {
  editor?.destroy?.();
  wrapper?.remove();
}

export function editor_read() {
  return editor.state.doc.toString();
}

export function editor_result() {
  const result = editor_read();
  editor_close();
  return result;
}

/**
 * @param {string}  content
 * @returns {EditorState}
 */
function state(content) {
  const state = EditorState.create({
    doc: content || "",
    extensions: [
      markdown({
        base: markdownLanguage,
        //codeLanguages: languages
      }),
      //keymap.of([indentWithTab, ...defaultKeymap]),
      EditorView.lineWrapping,
      EditorView.updateListener.of((v) => {
        if (v.docChanged) {
          //console.log("[UP]");
        }
      }),
      ...editorThemeExtensions(),
    ],
  });
  return state;
}
