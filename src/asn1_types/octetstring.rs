use crate::CheckDerConstraints;
use crate::{Any, Class, Error, Header, Length, Result, SerializeResult, Tag, Tagged, ToDer};
use std::borrow::Cow;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct OctetString<'a> {
    data: Cow<'a, [u8]>,
}

impl<'a> OctetString<'a> {
    pub const fn new(s: &'a [u8]) -> Self {
        OctetString {
            data: Cow::Borrowed(s),
        }
    }
}

impl<'a> AsRef<[u8]> for OctetString<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for OctetString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<OctetString<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        Ok(OctetString {
            data: any.into_cow(),
        })
    }
}

impl<'a> CheckDerConstraints for OctetString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for OctetString<'a> {
    const TAG: Tag = Tag::OctetString;
}

impl ToDer for OctetString<'_> {
    fn to_der_len(&self) -> Result<usize> {
        let header = Header::new(
            Class::Universal,
            0,
            Self::TAG,
            Length::Definite(self.data.len()),
        );
        Ok(header.to_der_len()? + self.data.len())
    }

    fn to_der(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let header = Header::new(
            Class::Universal,
            0,
            Self::TAG,
            Length::Definite(self.data.len()),
        );
        let sz = header.to_der(writer)?;
        let sz = sz + writer.write(&self.data)?;
        Ok(sz)
    }
}

impl<'a> TryFrom<Any<'a>> for &'a [u8] {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<&'a [u8]> {
        any.tag().assert_eq(Self::TAG)?;
        let s = OctetString::try_from(any)?;
        match s.data {
            Cow::Borrowed(s) => Ok(s),
            Cow::Owned(_) => Err(Error::LifetimeError),
        }
    }
}

impl<'a> CheckDerConstraints for &'a [u8] {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for &'a [u8] {
    const TAG: Tag = Tag::OctetString;
}

impl ToDer for &'_ [u8] {
    fn to_der_len(&self) -> Result<usize> {
        let header = Header::new(Class::Universal, 0, Self::TAG, Length::Definite(self.len()));
        Ok(header.to_der_len()? + self.len())
    }

    fn to_der(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let header = Header::new(Class::Universal, 0, Self::TAG, Length::Definite(self.len()));
        let sz = header.to_der(writer)?;
        let sz = sz + writer.write(self)?;
        Ok(sz)
    }
}
