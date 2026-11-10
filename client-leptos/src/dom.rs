use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, BlobPropertyBag, Element, EventTarget, File, HtmlAnchorElement, HtmlElement,
    HtmlInputElement, Node, Url,
    js_sys::{self},
};

pub fn get_window() -> Option<web_sys::Window> {
    web_sys::window()
}

pub fn get_location() -> Option<web_sys::Location> {
    get_window().map(|w| w.location())
}

pub fn get_document() -> Option<web_sys::Document> {
    get_window().and_then(|w| w.document())
}

pub fn get_element_from_node(el: Option<Node>) -> Option<Element> {
    el.map(|el| el.dyn_into::<Element>().ok()).and_then(|el| el)
}

pub fn html_element_from_target(target: Option<EventTarget>) -> Option<HtmlElement> {
    target
        .map(|target| JsValue::from(target).dyn_ref::<HtmlElement>().cloned())
        .and_then(|t| t)
}

pub fn html_input_from_target(target: Option<EventTarget>) -> Option<HtmlInputElement> {
    target
        .map(|target| JsValue::from(target).dyn_ref::<HtmlInputElement>().cloned())
        .and_then(|t| t)
}

pub fn get_html_element(el: Option<Element>) -> Option<HtmlElement> {
    el.map(|el| el.dyn_into::<HtmlElement>().ok())
        .and_then(|el| el)
}

pub fn query_selector_all(selectors: &str) -> Vec<HtmlElement> {
    let mut list: Vec<HtmlElement> = Vec::new();
    if let Some(d) = get_document() {
        let node_list = d.query_selector_all(selectors).unwrap_throw();
        for ind in 0..node_list.length() {
            if let Some(html_elem) = get_html_element(get_element_from_node(node_list.get(ind))) {
                list.push(html_elem);
            }
        }
    }
    list
}

pub fn storage_key() -> String {
    get_email()
        .replace(".ru", "")
        .replace("@", "")
        .replace(".", "")
        .replace("-", "")
}

pub fn get_email() -> String {
    get_location()
        .map(|l| {
            l.pathname().map_or(String::new(), |pathname| {
                pathname.split('/').collect::<Vec<_>>()[1].to_string()
            })
        })
        .and_then(|mail_box| test_email(&mail_box).then_some(mail_box))
        .unwrap_or_default()
}

fn test_email(email: &str) -> bool {
    let re: js_sys::RegExp = js_sys::RegExp::new("[a-z]+@[a-z-]+\\.[a-z]+", "");
    re.test(email)
}

pub fn local_storage_get(k: &str) -> Option<String> {
    get_window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|storage| storage.get_item(k).ok())
        .flatten()
}

pub fn local_storage_set(k: &str, v: &str) {
    if let Some(w) = get_window()
        && let Ok(Some(local_storage)) = w.local_storage()
    {
        local_storage.set_item(k, v).ok();
    }
}

pub fn local_storage_remove(k: &str) {
    if let Some(w) = get_window()
        && let Ok(Some(local_storage)) = w.local_storage()
    {
        local_storage.remove_item(k).ok();
    }
}

pub fn download_file((file_name, file_data): (String, Vec<u8>)) {
    if let Some(d) = get_window().and_then(|w| w.document())
        && let Ok(Some(link)) = d
            .create_element("a")
            .map(|el| el.dyn_into::<HtmlAnchorElement>().ok())
        && let Ok(data) = bytes_to_href(&file_data)
    {
        //let url = format!("/{API_DOWNLOAD}{file_id}");
        link.set_attribute("href", &data).ok();
        link.set_attribute("target", "blank").ok();
        link.set_attribute("download", &file_name).ok();
        if let Some(b) = d.body()
            && b.append_child(&link).is_ok()
        {
            link.click();
            link.remove();
        }
    }
}

// , mime_type: &str
pub fn bytes_to_href(data: &[u8]) -> Result<String, JsValue> {
    // 1. View the Vec<u8> as a Uint8Array
    let uint8_array = js_sys::Uint8Array::from(data);

    // 2. Wrap the array in an array sequence (required by Blob constructor)
    let array = js_sys::Array::new();
    array.push(&uint8_array);

    // 3. Create Blob properties (MIME type)
    let props = BlobPropertyBag::new();
    //props.type_(mime_type);

    // 4. Create the Blob
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &props)?;

    // 5. Create the object URL (href)
    let href = Url::create_object_url_with_blob(&blob)?;

    Ok(href)
}

pub async fn file_to_vec(file: &File) -> Result<Vec<u8>, JsValue> {
    // 1. Get a Promise for the ArrayBuffer from the file (Blob)
    let array_buffer_promise = file.array_buffer();

    // 2. Convert the JS Promise into a Rust Future and await it
    let array_buffer_jsvalue = JsFuture::from(array_buffer_promise).await?;

    // 3. Wrap the ArrayBuffer in a Uint8Array view
    let uint8_array = js_sys::Uint8Array::new(&array_buffer_jsvalue);

    // 4. Copy the JavaScript array data into a native Rust Vec<u8>
    let bytes = uint8_array.to_vec();

    Ok(bytes)
}
