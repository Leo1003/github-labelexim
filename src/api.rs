use serde::{Serialize, Deserialize};
use rgb::RGB;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    name: String,
    description: String,
    #[serde(with = "hex_color")]
    color: RGB<u8>,
}

mod hex_color {
    use rgb::RGB;
    use serde::{Deserializer, Deserialize, Serializer, de::{Unexpected, Error}};

    pub fn serialize<S>(
        color: &RGB<u8>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:02x}{:02x}{:02x}", color.r, color.g, color.b);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<RGB<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut buf = [0u8; 3];
        let s = <&str>::deserialize(deserializer)?;
        hex::decode_to_slice(s, &mut buf).map_err(|_| {
            Error::invalid_value(Unexpected::Str(s), &"a hexadecimal RGB color")
        })?;

        Ok(RGB {
            r: buf[0],
            g: buf[1],
            b: buf[2],
        })
    }
}
