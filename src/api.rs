use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use regex::Regex;
use rgb::RGB;

const GH_OWNER_REGEX: &str = r"(?P<owner>[[:alnum:]][[:alnum:]\-]+[[:alnum:]])";
const GH_REPO_REGEX: &str = r"(?P<repo>[[:alnum:]._\-]+?)";

lazy_static! {
    static ref GITHUB_REPO_HTTPS: Regex = {
        Regex::new(&format!(r"^https://github\.com/{}/{}(.git)?$", GH_OWNER_REGEX, GH_REPO_REGEX)).unwrap()
    };
    static ref GITHUB_REPO_GIT: Regex = {
        Regex::new(&format!(r"^git@github\.com:{}/{}(.git)?$", GH_OWNER_REGEX, GH_REPO_REGEX)).unwrap()
    };
    static ref GITHUB_REPO: Regex = {
        Regex::new(&format!(r"^{}/{}$", GH_OWNER_REGEX, GH_REPO_REGEX)).unwrap()
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    name: String,
    description: String,
    #[serde(with = "hex_color")]
    color: RGB<u8>,
}

pub fn parse_github_repo<'a> (s: &'a str) -> Option<(&'a str, &'a str)> {
    #[inline]
    fn _regcap<'a>(re: &Regex, s: &'a str) -> Option<(&'a str, &'a str)> {
        re.captures(s).map(|cap| {
            (cap.name("owner").unwrap().as_str(), cap.name("repo").unwrap().as_str())
        })
    }

    _regcap(&GITHUB_REPO_HTTPS, s).or_else(|| {
        _regcap(&GITHUB_REPO_GIT, s)
    }).or_else(|| {
        _regcap(&GITHUB_REPO, s)
    })
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
