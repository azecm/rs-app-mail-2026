use shared::{DataParams, DataResults, TEST_KEY, URL_TRANSPORT, decrypt_data, encrypt_data};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, js_sys::Uint8Array};

pub async fn transport_post(data: DataParams, session_id: Option<String>) -> DataResults {
    let hardware: [u8; 32] = device_data();

    match encrypt_data(&data, hardware, session_id, &TEST_KEY) {
        Ok(data) => match fetch_post(data).await {
            Ok(bytes) => match decrypt_data::<DataResults>(&bytes, &TEST_KEY) {
                Ok((results, _params)) => results,
                Err(err) => {
                    let error = format!("send_data::decrypt {err}");
                    log::error!("{error}");
                    DataResults::Error(error)
                }
            },
            Err(err) => {
                let error = format!("send_data::post {:?}", err);
                log::error!("{error}");
                DataResults::Error(error)
            }
        },
        Err(err) => {
            let error = format!("send_data::encrypt {err}");
            log::error!("{error}");
            DataResults::Error(error)
        }
    }
}

async fn fetch_post(bytes: Vec<u8>) -> Result<Vec<u8>, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);

    let js_array = Uint8Array::new_with_length(bytes.len() as u32);
    js_array.copy_from(&bytes);

    opts.set_body(&JsValue::from(js_array));

    let request = Request::new_with_str_and_init(URL_TRANSPORT, &opts)?;
    request
        .headers()
        .set("Content-Type", "application/octet-stream")?;

    let window = web_sys::window().expect("transport_post::window");
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = resp_value.dyn_into()?;

    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let uint8_array = Uint8Array::new(&array_buffer);
    let mut bytes = vec![0; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);

    Ok(bytes)
}

fn device_data() -> [u8; 32] {
    use std::cmp;
    let mut data_dig: Vec<usize> = Vec::new();
    let mut data_str: Vec<String> = Vec::new();
    if let Some(w) = crate::dom::get_window() {
        data_dig.push(w.device_pixel_ratio() as usize);
        if let Ok(screen) = w.screen() {
            let width = screen.width().unwrap_or_default() as usize;
            let height = screen.height().unwrap_or_default() as usize;
            data_dig.push(cmp::max(width, height));
            data_dig.push(cmp::min(width, height));
            data_dig.push(screen.pixel_depth().unwrap_or_default() as usize);
            data_dig.push(screen.color_depth().unwrap_or_default() as usize);
        }
        let navigator = w.navigator();
        data_dig.push(navigator.hardware_concurrency() as usize);
        data_dig.push(navigator.max_touch_points() as usize);
        data_str.push(navigator.app_code_name().unwrap_or_default());
        data_str.push(navigator.app_name());
        data_str.push(navigator.app_version().unwrap_or_default());
        data_str.push(navigator.language().unwrap_or_default());
        data_str.push(navigator.platform().unwrap_or_default());
        data_str.push(navigator.product());
        //data_str.push(navigator.user_agent().unwrap_or_default());
    }
    let mut hash = hmac_sha256::HMAC::new([]);
    hash.update(data_to_bytes(&data_dig));
    hash.update(data_to_bytes(&data_str));
    hash.finalize()
}

fn data_to_bytes<T>(data: &T) -> Vec<u8>
where
    T: bitcode::Encode,
{
    bitcode::encode(data)
}

/*
use crate::store::APP_SESSION;

fn get_session_id() -> Option<String> {
    let session_id = APP_SESSION.get_untracked();
    (!session_id.is_empty()).then_some(session_id)
}

pub fn transport_file(file_data: FileUpload, progress: RwSignal<usize>) {
    let hardware: [u8; 32] = device_data();

    let data = DataParams::Conn(DataConnParams::Files(DataFileParam::Upload(file_data)));

    match encrypt_data(&data, hardware, get_session_id(), &TEST_KEY) {
        Ok(data) => xhr_post(data, progress),
        Err(err) => {
            let error = format!("transport_file::encrypt {err}");
            log::error!("{error}");
            send_data_results(DataResults::Error(error));
        }
    }
}

fn xhr_post(bytes: Vec<u8>, progress: RwSignal<usize>) {
    if let Ok(xhr) = web_sys::XmlHttpRequest::new() {
        if xhr
            .open_with_async("POST", &format!("/{URL_TRANSPORT}"), true)
            .is_err()
        {
            return;
        }
        xhr.set_request_header("Content-Type", "application/octet-stream")
            .ok();

        if let Ok(upload) = xhr.upload() {
            let onprogress_callback =
                Closure::<dyn FnMut(_)>::new(move |e: web_sys::ProgressEvent| {
                    let value = (e.loaded() / e.total() * 1000.0).round() as usize;
                    progress.set(value);
                });
            upload.set_onprogress(Some(onprogress_callback.as_ref().unchecked_ref()));
            onprogress_callback.forget();
        }

        xhr.send_with_opt_u8_array(Some(&bytes)).ok();
        xhr.set_response_type(web_sys::XmlHttpRequestResponseType::Arraybuffer);

        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |_e: web_sys::ErrorEvent| {
            log::error!("[connect_files]");
            send_data_results(DataResults::Error(
                "Ошибка при отправке файла[1]".to_string(),
            ));
        });
        xhr.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let xhr_load = xhr.clone();
        let onload_callback = Closure::<dyn FnMut()>::new(move || {
            if let Ok(status) = xhr_load.status()
                && status > 199
                && status < 300
            {
                match xhr_load.response() {
                    Ok(data) => match data.dyn_into::<ArrayBuffer>() {
                        Ok(array_buffer) => {
                            let data = Uint8Array::new(&array_buffer).to_vec();
                            match decrypt_data::<DataResults>(&data, &TEST_KEY) {
                                Ok(data) => {
                                    send_data_results(data.0);
                                }
                                Err(err) => {
                                    send_data_results(DataResults::Error(format!(
                                        "Ошибка при отправке файла[1] {:?}",
                                        err
                                    )));
                                }
                            }
                        }
                        Err(err) => {
                            send_data_results(DataResults::Error(format!(
                                "Ошибка при отправке файла[2] {:?}",
                                err.as_string().unwrap_or_default()
                            )));
                        }
                    },
                    Err(err) => {
                        send_data_results(DataResults::Error(format!(
                            "Ошибка при отправке файла[3] {:?}",
                            err.as_string()
                        )));
                    }
                }
            }
        });
        xhr.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
        onload_callback.forget();
    }
}
*/
