use crate::datetime::decode_decimal;
use crate::{
    ASN1DateTime, ASN1TimeZone, Any, CheckDerConstraints, Error, Result, Tag, Tagged, ToDer,
};
#[cfg(feature = "datetime")]
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcTime(pub ASN1DateTime);

impl UtcTime {
    pub const fn new(datetime: ASN1DateTime) -> Self {
        UtcTime(datetime)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // X.680 section 43 defines a UniversalTime as a VisibleString restricted to:
        //
        // a) the six digits YYMMDD where YY is the two low-order digits of the Christian year, MM is the month
        // (counting January as 01), and DD is the day of the month (01 to 31); and
        // b) either:
        //   1) the four digits hhmm where hh is hour (00 to 23) and mm is minutes (00 to 59); or
        //   2) the six digits hhmmss where hh and mm are as in 1) above, and ss is seconds (00 to 59); and
        // c) either:
        //   1) the character Z ; or
        //   2) one of the characters + or - , followed by hhmm, where hh is hour and mm is minutes.
        //
        // XXX // RFC 5280 requires mandatory seconds and Z-normalized time zone
        let (year, month, day, hour, minute, rem) = match bytes {
            [year1, year2, mon1, mon2, day1, day2, hour1, hour2, min1, min2, rem @ ..] => {
                let year = decode_decimal(Self::TAG, *year1, *year2)?;
                let month = decode_decimal(Self::TAG, *mon1, *mon2)?;
                let day = decode_decimal(Self::TAG, *day1, *day2)?;
                let hour = decode_decimal(Self::TAG, *hour1, *hour2)?;
                let minute = decode_decimal(Self::TAG, *min1, *min2)?;
                (year, month, day, hour, minute, rem)
            }
            _ => return Err(Self::TAG.invalid_value("malformed time string (not yymmddhhmm)")),
        };
        if rem.is_empty() {
            return Err(Self::TAG.invalid_value("malformed time string"));
        }
        // check for seconds
        let (second, rem) = match rem {
            [sec1, sec2, rem @ ..] => {
                let second = decode_decimal(Self::TAG, *sec1, *sec2)?;
                (second, rem)
            }
            _ => (0, rem),
        };
        if month > 12 || day > 31 || hour > 23 || minute > 59 || second > 59 {
            return Err(Self::TAG.invalid_value("time components with invalid values"));
        }
        if rem.is_empty() {
            return Err(Self::TAG.invalid_value("malformed time string"));
        }
        let tz = match rem {
            [b'Z'] => ASN1TimeZone::Z,
            [b'+', h1, h2, m1, m2] => {
                let hh = decode_decimal(Self::TAG, *h1, *h2)?;
                let mm = decode_decimal(Self::TAG, *m1, *m2)?;
                ASN1TimeZone::Offset(1, hh, mm)
            }
            [b'-', h1, h2, m1, m2] => {
                let hh = decode_decimal(Self::TAG, *h1, *h2)?;
                let mm = decode_decimal(Self::TAG, *m1, *m2)?;
                ASN1TimeZone::Offset(-1, hh, mm)
            }
            _ => return Err(Self::TAG.invalid_value("malformed time string: no time zone")),
        };
        Ok(UtcTime(ASN1DateTime::new(
            year as u32,
            month,
            day,
            hour,
            minute,
            second,
            None,
            tz,
        )))
        // match *bytes {
        //     [year1, year2, mon1, mon2, day1, day2, hour1, hour2, min1, min2, sec1, sec2, b'Z'] => {
        //         let year = decode_decimal(Self::TAG, year1, year2)?;
        //         let month = decode_decimal(Self::TAG, mon1, mon2)?;
        //         let day = decode_decimal(Self::TAG, day1, day2)?;
        //         let hour = decode_decimal(Self::TAG, hour1, hour2)?;
        //         let minute = decode_decimal(Self::TAG, min1, min2)?;
        //         let second = decode_decimal(Self::TAG, sec1, sec2)?;

        //         // RFC 5280 rules for interpreting the year
        //         let year = if year >= 50 { year + 1900 } else { year + 2000 };

        //         Ok(UtcTime::new(year, month, day, hour, minute, second))
        //     }
        //     _ => Err(Error::InvalidValue),
        // }
    }

    /// Return a ISO 8601 combined date and time with time zone.
    #[cfg(feature = "datetime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "datetime")))]
    pub fn utc_datetime(&self) -> DateTime<Utc> {
        let dt = &self.0;
        // XXX Utc only if Z
        Utc.ymd(dt.year as i32, dt.month as u32, dt.day as u32)
            .and_hms(dt.hour as u32, dt.minute as u32, dt.second as u32)
    }

    /// Returns the number of non-leap seconds since the midnight on January 1, 1970.
    #[cfg(feature = "datetime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "datetime")))]
    pub fn timestamp(&self) -> i64 {
        let dt = &self.0;
        let d = NaiveDate::from_ymd(dt.year as i32, dt.month as u32, dt.day as u32);
        let t = NaiveTime::from_hms(dt.hour as u32, dt.minute as u32, dt.second as u32);
        let ndt = NaiveDateTime::new(d, t);
        // XXX offset?
        ndt.timestamp()
    }
}

impl<'a> TryFrom<Any<'a>> for UtcTime {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<UtcTime> {
        any.tag().assert_eq(Self::TAG)?;
        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_visible(b: &u8) -> bool {
            0x20 <= *b && *b <= 0x7f
        }
        if !any.data.iter().all(is_visible) {
            return Err(Error::StringInvalidCharset);
        }

        UtcTime::from_bytes(&any.data)
    }
}

impl fmt::Display for UtcTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dt = &self.0;
        match dt.tz {
            ASN1TimeZone::Z | ASN1TimeZone::Undefined => write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} Z",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
            ),
            ASN1TimeZone::Offset(sign, hh, mm) => {
                let s = if sign > 0 { '+' } else { '-' };
                write!(
                    f,
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}{:02}{:02}",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second, s, hh, mm
                )
            }
        }
    }
}

impl CheckDerConstraints for UtcTime {
    fn check_constraints(_any: &Any) -> Result<()> {
        Ok(())
    }
}

impl Tagged for UtcTime {
    const TAG: Tag = Tag::UtcTime;
}

impl ToDer for UtcTime {
    fn to_der_len(&self) -> Result<usize> {
        // data:
        // - 6 bytes for YYMMDD
        // - 6 for hhmmss in DER (X.690 section 11.8.2)
        // - 1 for the character Z in DER (X.690 section 11.8.1)
        // data length: 13
        //
        // thus, length will always be on 1 byte (short length) and
        // class+structure+tag also on 1
        //
        // total: 15 = 1 (class+structured+tag) + 1 (length) + 13
        Ok(15)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
        // see above for length value
        writer.write(&[Self::TAG.0 as u8, 13]).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
        let _ = write!(
            writer,
            "{:02}{:02}{:02}{:02}{:02}{:02}Z",
            self.0.year, self.0.month, self.0.day, self.0.hour, self.0.minute, self.0.second,
        )?;
        // write_fmt returns (), see above for length value
        Ok(13)
    }
}
