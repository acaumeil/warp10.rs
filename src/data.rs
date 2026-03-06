use core::fmt::Write as _;

use chrono::{DateTime, FixedOffset};
#[cfg(feature = "json")]
use serde::Serialize;

#[cfg(feature = "json")]
use crate::error;

fn url_encode(input: &str) -> String {
    let mut code = String::new();
    code.extend(percent_encoding::utf8_percent_encode(
        input,
        percent_encoding::NON_ALPHANUMERIC,
    ));
    code
}

pub trait Warp10Serializable {
    fn warp10_serialize(&self) -> String;
}

pub type Int = i32;
pub type Long = i64;
pub type Double = f64;
pub type Boolean = bool;

#[derive(Debug, Clone, PartialEq)]
pub struct HHCode {
    pub lat: Double,
    pub lon: Double,
}

impl HHCode {
    #[must_use]
    #[inline]
    pub const fn new(lat: Double, lon: Double) -> Self {
        Self { lat, lon }
    }
}

#[cfg(feature = "warp10_version_2_1")]
#[derive(Debug, Clone, PartialEq)]
pub struct Quaternions {
    pub w: Double,
    pub x: Double,
    pub y: Double,
    pub z: Double,
}

#[cfg(feature = "warp10_version_2_1")]
impl Quaternions {
    #[must_use]
    #[inline]
    pub const fn new(w: Double, x: Double, y: Double, z: Double) -> Self {
        Self { w, x, y, z }
    }
}

#[cfg(feature = "warp10_version_2_1")]
#[derive(Debug, Clone, PartialEq)]
pub enum MultiValueKind {
    UnamedGTS(UnamedGeoTimeSeries),
    Value(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnamedGeoTimeSeries {
    pub timestamp: DateTime<FixedOffset>,
    pub lat: Option<Double>,
    pub lon: Option<Double>,
    pub elev: Option<Long>,
    pub value: Value,
}

impl UnamedGeoTimeSeries {
    #[must_use]
    #[inline]
    pub const fn new(timestamp: DateTime<FixedOffset>, value: Value) -> Self {
        Self::for_ts(timestamp, value)
    }

    #[must_use]
    #[inline]
    pub const fn for_ts(timestamp: DateTime<FixedOffset>, value: Value) -> Self {
        Self {
            timestamp,
            lat: None,
            lon: None,
            elev: None,
            value,
        }
    }

    #[must_use]
    #[inline]
    pub const fn with_geo(mut self, lat: Double, lon: Double, elev: Option<Long>) -> Self {
        self.lat = Some(lat);
        self.lon = Some(lon);
        self.elev = elev;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_elev(mut self, elev: Long) -> Self {
        self.elev = Some(elev);
        self
    }

    #[must_use]
    #[inline]
    pub fn validate(self) -> bool {
        self.lat.is_some() == self.lon.is_some()
    }
}

impl Warp10Serializable for UnamedGeoTimeSeries {
    fn warp10_serialize(&self) -> String {
        let mut coord = String::new();
        if let (Some(lat), Some(lon)) = (self.lat, self.lon) {
            let _ = write!(coord, "{lat}:{lon}");
        }
        if let Some(elev) = self.elev {
            coord.push('/');
            coord.push_str(&elev.to_string());
        }
        if coord.is_empty() {
            format!(
                "{}/{}",
                self.timestamp.timestamp_micros(),
                self.value.warp10_serialize()
            )
        } else {
            format!(
                "{}/{}/{}",
                self.timestamp.timestamp_micros(),
                coord,
                self.value.warp10_serialize()
            )
        }
    }
}

#[cfg(feature = "warp10_version_2_1")]
#[derive(Debug, Clone, PartialEq)]
pub struct MultiValue {
    pub data: Vec<MultiValueKind>,
    pub compressed: bool,
}

#[cfg(feature = "warp10_version_2_1")]
impl MultiValue {
    #[must_use]
    #[inline]
    pub const fn new(data: Vec<MultiValueKind>) -> Self {
        Self {
            data,
            compressed: true,
        }
    }

    #[must_use]
    #[inline]
    pub const fn no_compression(mut self) -> Self {
        self.compressed = false;
        self
    }
}

#[cfg(feature = "warp10_version_2_1")]
impl Warp10Serializable for MultiValue {
    fn warp10_serialize(&self) -> String {
        let values = self
            .data
            .iter()
            .map(|data| match *data {
                MultiValueKind::Value(ref value) => value.warp10_serialize(),
                MultiValueKind::UnamedGTS(ref gts) => gts.warp10_serialize(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!("[{}{}]", if self.compressed { "!" } else { "" }, values)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(Int),
    Long(Long),
    Double(Double),
    Boolean(Boolean),
    String(String),
    HHCode(HHCode),
    #[cfg(feature = "warp10_version_2_1")]
    Quaternions(Quaternions),
    #[cfg(feature = "warp10_version_2_1")]
    BinaryB64(String),
    #[cfg(feature = "warp10_version_2_1")]
    BinaryHex(String),
    #[cfg(feature = "warp10_version_2_1")]
    MultiValue(MultiValue),
}

impl Warp10Serializable for Value {
    #[inline]
    fn warp10_serialize(&self) -> String {
        match *self {
            Self::Int(i) => i.to_string(),
            Self::Long(l) => l.to_string(),
            Self::Double(d) => d.to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::String(ref s) => format!("'{}'", url_encode(s)),
            Self::HHCode(ref hhc) => format!("HH:{}:{}", hhc.lat, hhc.lon),
            #[cfg(feature = "warp10_version_2_1")]
            Self::Quaternions(ref q) => format!("Q:{}:{}:{}:{}", q.w, q.x, q.y, q.z),
            #[cfg(feature = "warp10_version_2_1")]
            Self::BinaryB64(ref b64) => format!("b64:{b64}"),
            #[cfg(feature = "warp10_version_2_1")]
            Self::BinaryHex(ref hex) => format!("hex:{hex}"),
            #[cfg(feature = "warp10_version_2_1")]
            Self::MultiValue(ref mv) => mv.warp10_serialize(),
        }
    }
}

impl From<Int> for Value {
    #[inline]
    fn from(i: Int) -> Self {
        Self::Int(i)
    }
}

impl From<Long> for Value {
    #[inline]
    fn from(l: Long) -> Self {
        Self::Long(l)
    }
}

impl From<Double> for Value {
    #[inline]
    fn from(d: Double) -> Self {
        Self::Double(d)
    }
}

impl From<Boolean> for Value {
    #[inline]
    fn from(b: Boolean) -> Self {
        Self::Boolean(b)
    }
}

impl From<&str> for Value {
    #[inline]
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for Value {
    #[inline]
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<HHCode> for Value {
    #[inline]
    fn from(hhc: HHCode) -> Self {
        Self::HHCode(hhc)
    }
}

#[cfg(feature = "warp10_version_2_1")]
impl From<Quaternions> for Value {
    #[inline]
    fn from(q: Quaternions) -> Self {
        Self::Quaternions(q)
    }
}

#[cfg(feature = "warp10_version_2_1")]
impl From<MultiValue> for Value {
    #[inline]
    fn from(mv: MultiValue) -> Self {
        Self::MultiValue(mv)
    }
}

impl Value {
    #[cfg(feature = "json")]
    #[inline]
    pub fn try_from<T: Serialize>(obj: &T) -> error::Result<Self> {
        Ok(Self::String(serde_json::to_string(obj)?))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeoValue {
    pub lat: Double,
    pub lon: Double,
    pub elev: Option<Long>,
}

impl GeoValue {
    #[must_use]
    #[inline]
    pub const fn new(lat: Double, lon: Double, elev: Option<Long>) -> Self {
        Self { lat, lon, elev }
    }
}

impl Warp10Serializable for GeoValue {
    #[inline]
    fn warp10_serialize(&self) -> String {
        format!(
            "{}:{}/{}",
            self.lat,
            self.lon,
            self.elev.map(|e| e.to_string()).unwrap_or_default()
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: String,
    pub value: String,
}

impl Label {
    #[must_use]
    #[inline]
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: value.to_owned(),
        }
    }
}

impl Warp10Serializable for Label {
    #[inline]
    fn warp10_serialize(&self) -> String {
        format!("{}={}", url_encode(&self.name), url_encode(&self.value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub date: Option<DateTime<FixedOffset>>,
    pub geo: Option<GeoValue>,
    pub name: String,
    pub labels: Vec<Label>,
    pub value: Value,
}

impl Data {
    #[must_use]
    #[inline]
    pub const fn new(
        date: DateTime<FixedOffset>,
        geo: Option<GeoValue>,
        name: String,
        labels: Vec<Label>,
        value: Value,
    ) -> Self {
        Self {
            date: Some(date),
            geo,
            name,
            labels,
            value,
        }
    }

    #[must_use]
    #[inline]
    pub const fn new_without_time(
        geo: Option<GeoValue>,
        name: String,
        labels: Vec<Label>,
        value: Value,
    ) -> Self {
        Self {
            date: None,
            geo,
            name,
            labels,
            value,
        }
    }
}

impl Warp10Serializable for Data {
    fn warp10_serialize(&self) -> String {
        let geo = self
            .geo
            .as_ref()
            .map(Warp10Serializable::warp10_serialize)
            .unwrap_or_else(|| "/".to_owned());
        let labels =
            self.labels
                .iter()
                .map(|label| label.warp10_serialize())
                .fold(String::new(), |acc, cur| {
                    if acc.is_empty() {
                        cur
                    } else {
                        (acc + ",") + &cur
                    }
                });

        match self.date {
            Some(date) => {
                let date_ms = date.timestamp_micros();
                format!(
                    "{}/{} {}{{{}}} {}",
                    date_ms,
                    geo,
                    url_encode(&self.name),
                    labels,
                    self.value.warp10_serialize()
                )
            }
            None => {
                // In this case the warp10 instance will put the same timestamp for the data point has
                // the ingestion one.
                format!(
                    "/{} {}{{{}}} {}",
                    geo,
                    url_encode(&self.name),
                    labels,
                    self.value.warp10_serialize()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeDelta;

    #[test]
    fn serialize_int() {
        assert_eq!(Value::Int(42).warp10_serialize(), "42");
    }

    #[test]
    fn serialize_long() {
        assert_eq!(Value::Long(42).warp10_serialize(), "42");
    }

    #[test]
    fn serialize_double() {
        assert_eq!(Value::Double(42.66).warp10_serialize(), "42.66");
    }

    #[test]
    fn serialize_boolean() {
        assert_eq!(Value::Boolean(true).warp10_serialize(), "true");
        assert_eq!(Value::Boolean(false).warp10_serialize(), "false");
    }

    #[test]
    fn serialize_string() {
        assert_eq!(
            Value::String("foobar".to_owned()).warp10_serialize(),
            "'foobar'"
        );

        assert_eq!(
            Value::String("hello warp10".to_owned()).warp10_serialize(),
            "'hello%20warp10'"
        );
    }

    #[test]
    fn serialize_hh_code() {
        assert_eq!(
            Value::HHCode(HHCode::new(42.66, 32.85)).warp10_serialize(),
            "HH:42.66:32.85"
        );
    }

    #[cfg(feature = "warp10_version_2_1")]
    #[test]
    fn serialize_quaternions() {
        assert_eq!(
            Value::Quaternions(Quaternions::new(
                0.7071067811865475,
                0.408248290463863,
                0.408248290463863,
                0.408248290463863
            ))
            .warp10_serialize(),
            "Q:0.7071067811865475:0.408248290463863:0.408248290463863:0.408248290463863"
        );
    }

    #[cfg(feature = "warp10_version_2_1")]
    #[test]
    fn serialize_binary_b64() {
        assert_eq!(
            Value::BinaryB64("aGVsbG8gV2FycCAxMAo=".to_owned()).warp10_serialize(),
            "b64:aGVsbG8gV2FycCAxMAo="
        );
    }

    #[cfg(feature = "warp10_version_2_1")]
    #[test]
    fn serialize_binary_hex() {
        assert_eq!(
            Value::BinaryHex("68656c6c6f2057617270203130".to_owned()).warp10_serialize(),
            "hex:68656c6c6f2057617270203130"
        );
    }

    #[test]
    fn serialize_unamed_geo_time_series() {
        assert_eq!(
            UnamedGeoTimeSeries::new(
                (chrono::DateTime::UNIX_EPOCH + TimeDelta::new(25, 123456789).unwrap())
                    .fixed_offset(),
                Value::Int(42)
            )
            .warp10_serialize(),
            "25123456/42"
        );

        assert_eq!(
            UnamedGeoTimeSeries::new(
                (chrono::DateTime::UNIX_EPOCH + TimeDelta::new(25, 123456789).unwrap())
                    .fixed_offset(),
                Value::Int(42)
            )
            .with_geo(42.66, 32.85, None)
            .warp10_serialize(),
            "25123456/42.66:32.85/42"
        );

        assert_eq!(
            UnamedGeoTimeSeries::new(
                (chrono::DateTime::UNIX_EPOCH + TimeDelta::new(25, 123456789).unwrap())
                    .fixed_offset(),
                Value::Int(42)
            )
            .with_elev(10)
            .warp10_serialize(),
            "25123456//10/42"
        );

        assert_eq!(
            UnamedGeoTimeSeries::new(
                (chrono::DateTime::UNIX_EPOCH + TimeDelta::new(25, 123456789).unwrap())
                    .fixed_offset(),
                Value::Int(42)
            )
            .with_geo(42.66, 32.85, Some(10))
            .warp10_serialize(),
            "25123456/42.66:32.85/10/42"
        );
    }

    #[cfg(feature = "warp10_version_2_1")]
    #[test]
    fn serialize_multi_value() {
        assert_eq!(
            Value::MultiValue(MultiValue::new(vec![
                MultiValueKind::Value(Value::Int(42)),
                MultiValueKind::UnamedGTS(
                    UnamedGeoTimeSeries::new(
                        (chrono::DateTime::UNIX_EPOCH + TimeDelta::new(25, 123456789).unwrap())
                            .fixed_offset(),
                        Value::Int(42)
                    )
                    .with_geo(42.66, 32.85, Some(10))
                )
            ]))
            .warp10_serialize(),
            "[!42 25123456/42.66:32.85/10/42]"
        );

        assert_eq!(
            Value::MultiValue(
                MultiValue::new(vec![
                    MultiValueKind::Value(Value::Int(42)),
                    MultiValueKind::Value(Value::Double(8.1))
                ])
                .no_compression()
            )
            .warp10_serialize(),
            "[42 8.1]"
        );
    }

    #[test]
    fn serialize_geo() {
        assert_eq!(
            GeoValue::new(42.66, 32.85, None).warp10_serialize(),
            "42.66:32.85/"
        );
        assert_eq!(
            GeoValue::new(42.66, 32.85, Some(10)).warp10_serialize(),
            "42.66:32.85/10"
        );
    }

    #[test]
    fn serialize_label() {
        assert_eq!(
            Label::new("name 1", "\u{51c4}\u{3044} value 2").warp10_serialize(),
            "name%201=%E5%87%84%E3%81%84%20value%202"
        );
    }

    #[test]
    fn serialize_data() {
        assert_eq!(
            Data::new(
                (chrono::DateTime::UNIX_EPOCH + TimeDelta::new(25, 123456789).unwrap())
                    .fixed_offset(),
                None,
                "original name".to_owned(),
                vec![
                    Label::new("label1", "value1"),
                    Label::new("label 2", "value 2"),
                ],
                Value::String("foobar".to_owned())
            )
            .warp10_serialize(),
            "25123456// original%20name{label1=value1,label%202=value%202} 'foobar'"
        );
        assert_eq!(
            Data::new(
                (chrono::DateTime::UNIX_EPOCH + TimeDelta::new(25, 123456789).unwrap())
                    .fixed_offset(),
                Some(GeoValue::new(42.66, 32.85, Some(10))),
                "original name".to_owned(),
                vec![
                    Label::new("label1", "value1"),
                    Label::new("label 2", "value 2"),
                ],
                Value::String("foobar".to_owned())
            )
            .warp10_serialize(),
            "25123456/42.66:32.85/10 original%20name{label1=value1,label%202=value%202} 'foobar'"
        );
        assert_eq!(
            Data::new_without_time(
                Some(GeoValue::new(42.66, 32.85, Some(10))),
                "original name".to_owned(),
                vec![
                    Label::new("label1", "value1"),
                    Label::new("label 2", "value 2"),
                ],
                Value::String("foobar".to_owned())
            )
            .warp10_serialize(),
            "/42.66:32.85/10 original%20name{label1=value1,label%202=value%202} 'foobar'"
        );
    }

    #[test]
    #[cfg(feature = "json")]
    fn serialize_structure_into_json_string() -> Result<(), crate::error::Error> {
        assert_eq!(
            Value::try_from(&vec![""])?,
            Value::String("[\"\"]".to_string())
        );

        let mut map = std::collections::HashMap::new();

        map.insert("baz", "qux'");

        assert_eq!(
            Value::try_from(&map)?,
            Value::String("{\"baz\":\"qux'\"}".to_string()),
        );

        assert_eq!(
            Data::new_without_time(
                Some(GeoValue::new(42.66, 32.85, Some(10))),
                "original name".to_string(),
                vec![
                    Label::new("label1", "value1"),
                    Label::new("label 2", "value 2"),
                ],
                Value::try_from(&map)?,
            )
                .warp10_serialize(),
            "/42.66:32.85/10 original%20name{label1=value1,label%202=value%202} '{\"baz\":\"qux'\"}'"
        );

        Ok(())
    }
}
