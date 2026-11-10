import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorThemeExtensions } from "./style.js";
import "./css/themes.css";

/* npm run build */

console.log("editor-module");

/** @type HTMLDivElement */
let wrapper;
/** @type EditorView */
let editor;

/**
 * @param {string}  content
 */
export function editor_open(content) {
  wrapper = document.createElement("div");
  wrapper.id = "editor";
  document.body.appendChild(wrapper);

  editor = new EditorView({
    state: state(content),
    parent: document.querySelector("#editor"),
  });
}

export function editor_close() {
  const result = editor.state.doc.toString();
  editor?.destroy?.();
  wrapper?.remove();
  return result;
}

/**
 * @param {string}  content
 * @returns {EditorState}
 */
function state(content) {
  const state = EditorState.create({
    doc: content || "# markon\n\nStart typing...",
    extensions: [
      markdown({
        base: markdownLanguage,
        //codeLanguages: languages
      }),
      //keymap.of([indentWithTab, ...defaultKeymap]),
      EditorView.lineWrapping,
      EditorView.updateListener.of((v) => {
        if (v.docChanged) {
          console.log("[UP]");
        }
      }),
      ...editorThemeExtensions(),
    ],
  });
  return state;
}
