use bytes::Bytes;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct EncodedValue {
    data: Bytes,
}

impl From<Bytes> for EncodedValue {
    fn from(data: Bytes) -> Self {
        Self {
            data
        }
    }
}