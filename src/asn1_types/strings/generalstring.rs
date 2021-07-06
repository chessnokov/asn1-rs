use crate::asn1_string;
use crate::{Error, Result};

asn1_string!(GeneralString);

impl<'a> GeneralString<'a> {
    fn test_string_charset(i: &[u8]) -> Result<()> {
        if !i.iter().all(u8::is_ascii) {
            return Err(Error::StringInvalidCharset);
        }
        Ok(())
    }
}