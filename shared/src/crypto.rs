use crate::{Transport, TransportContainer, TransportParams, VERSION};
#[allow(deprecated)]
use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, AeadCore, KeyInit, OsRng, generic_array::GenericArray},
};

fn decrypt(data: Transport, key: &[u8; 32]) -> anyhow::Result<Vec<u8>> {
    let key: &Key<Aes256Gcm> = key.into();
    let cipher = Aes256Gcm::new(key);
    #[allow(deprecated)]
    let nonce = GenericArray::from_slice(&data.1);
    cipher
        .decrypt(nonce, data.0.as_ref())
        .map_err(|e| anyhow::anyhow!("crypto::decrypt {e:?}"))
}

fn encrypt(data: Vec<u8>, key: &[u8; 32]) -> anyhow::Result<Transport> {
    let key: &Key<Aes256Gcm> = key.into();
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let data = cipher
        .encrypt(&nonce, data.as_ref())
        .map_err(|e| anyhow::anyhow!("crypto::encrypt {e:?}"))?;
    //#[allow(deprecated)]
    Ok(Transport(data, nonce.to_vec()))
}

fn data_to_bytes<T>(data: &T) -> Vec<u8>
where
    T: bitcode::Encode,
{
    bitcode::encode(data)
}

fn data_from_bytes<T>(data: &[u8]) -> anyhow::Result<T>
where
    T: for<'a> bitcode::Decode<'a>,
{
    bitcode::decode(data).map_err(|e| anyhow::anyhow!("bitcode::decode: {}", e))
}

/*
pub fn decrypt_data<T>(data: &[u8], key: &[u8; 32]) -> anyhow::Result<T>
where
    T: for<'a> bitcode::Decode<'a>,
{
    data_from_bytes(data).and_then(|d| decrypt(d, key).and_then(|d| data_from_bytes(&d)))
}

pub fn encrypt_data<T>(data: &T, key: &[u8; 32]) -> anyhow::Result<Vec<u8>>
where
    T: bitcode::Encode,
{
    encrypt(data_to_bytes(data), key).map(|d| data_to_bytes(&d))
}


pub fn encrypt_to_bytes<T>(data: T) -> anyhow::Result<Bytes>
where
    T: bitcode::Encode,
{
    encrypt_data(&data, &TEST_KEY).map(Bytes::from)
}
*/
pub fn decrypt_data<T>(data: &[u8], key: &[u8; 32]) -> anyhow::Result<(T, TransportParams)>
where
    T: for<'a> bitcode::Decode<'a>,
{
    data_from_bytes(data).and_then(|d| {
        decrypt(d, key).and_then(|d| {
            match data_from_bytes::<TransportContainer>(&d) {
                Ok(data) => {
                    match data_from_bytes(&data.data) {
                        // data.hardware, data.session_id, data.version
                        Ok(transport) => Ok((
                            transport,
                            TransportParams {
                                hardware: data.param.hardware,
                                session: data.param.session,
                                version: data.param.version,
                            },
                        )),
                        Err(err) => Err(anyhow::anyhow!("data_from_bytes::err {err:?}")),
                    }
                }
                Err(err) => Err(anyhow::anyhow!("decrypt_data::err -> {err:?}")),
            }
        })
    })
}

pub fn encrypt_data<T>(
    data: &T,
    hardware: [u8; 32],
    session_id: Option<String>,
    key: &[u8; 32],
) -> anyhow::Result<Vec<u8>>
where
    T: bitcode::Encode,
{
    encrypt(
        data_to_bytes(&TransportContainer {
            data: data_to_bytes(data),
            param: TransportParams {
                hardware,
                session: session_id,
                version: VERSION,
            },
        }),
        key,
    )
    .map(|d| data_to_bytes(&d))
}
