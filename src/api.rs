use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{Client, Error as ReqError, Request};
use rgb::RGB;
use serde::{Deserialize, Serialize};

const GH_OWNER_REGEX: &str = r"(?P<owner>[[:alnum:]][[:alnum:]\-]+[[:alnum:]])";
const GH_REPO_REGEX: &str = r"(?P<repo>[[:alnum:]._\-]+?)";

lazy_static! {
    static ref GITHUB_REPO_HTTPS: Regex = {
        Regex::new(&format!(
            r"^https://github\.com/{}/{}(.git)?$",
            GH_OWNER_REGEX, GH_REPO_REGEX
        ))
        .unwrap()
    };
    static ref GITHUB_REPO_GIT: Regex = {
        Regex::new(&format!(
            r"^git@github\.com:{}/{}(.git)?$",
            GH_OWNER_REGEX, GH_REPO_REGEX
        ))
        .unwrap()
    };
    static ref GITHUB_REPO: Regex =
        { Regex::new(&format!(r"^{}/{}$", GH_OWNER_REGEX, GH_REPO_REGEX)).unwrap() };
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    name: String,
    description: String,
    #[serde(with = "hex_color")]
    color: RGB<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LabelUpdate<'l> {
    new_name: Option<&'l str>,
    description: &'l str,
    #[serde(with = "hex_color")]
    color: &'l RGB<u8>,
}

impl<'l> LabelUpdate<'l> {
    pub fn with_name(label: &'l Label) -> Self {
        LabelUpdate {
            new_name: Some(&label.name),
            description: &label.description,
            color: &label.color,
        }
    }

    pub fn without_name(label: &'l Label) -> Self {
        LabelUpdate {
            new_name: None,
            description: &label.description,
            color: &label.color,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GithubClient {
    token_hdr: String,
    client: Client,
}

impl GithubClient {
    fn new(token: &str) -> Self {
        GithubClient {
            token_hdr: format!("token {}", token),
            client: Client::new(),
        }
    }
}

impl GithubClient {
    pub async fn check_token(&self) -> Result<(), ReqError> {
        self.client
            .get("https://api.github.com/user")
            .header(reqwest::header::AUTHORIZATION, &self.token_hdr)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_labels(&self, owner: &str, repo: &str) -> Result<Vec<Label>, ReqError> {
        self.client
            .get(&format!("/repos/{}/{}/labels", owner, repo))
            .header(reqwest::header::AUTHORIZATION, &self.token_hdr)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Label>>()
            .await
    }

    pub async fn new_label(&self, owner: &str, repo: &str, label: &Label) -> Result<(), ReqError> {
        self.client
            .post(&format!("/repos/{}/{}/labels", owner, repo))
            .header(reqwest::header::AUTHORIZATION, &self.token_hdr)
            .json(label)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn update_label(
        &self,
        owner: &str,
        repo: &str,
        label: &Label,
    ) -> Result<(), ReqError> {
        self.client
            .patch(&format!("/repos/{}/{}/labels/{}", owner, repo, &label.name))
            .header(reqwest::header::AUTHORIZATION, &self.token_hdr)
            .json(&LabelUpdate::without_name(label))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn update_label_with_name(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        label: &Label,
    ) -> Result<(), ReqError> {
        self.client
            .patch(&format!("/repos/{}/{}/labels/{}", owner, repo, name))
            .header(reqwest::header::AUTHORIZATION, &self.token_hdr)
            .json(&LabelUpdate::with_name(label))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

pub fn parse_github_repo<'a>(s: &'a str) -> Option<(&'a str, &'a str)> {
    #[inline]
    fn _regcap<'a>(re: &Regex, s: &'a str) -> Option<(&'a str, &'a str)> {
        re.captures(s).map(|cap| {
            (
                cap.name("owner").unwrap().as_str(),
                cap.name("repo").unwrap().as_str(),
            )
        })
    }

    _regcap(&GITHUB_REPO_HTTPS, s)
        .or_else(|| _regcap(&GITHUB_REPO_GIT, s))
        .or_else(|| _regcap(&GITHUB_REPO, s))
}

mod hex_color {
    use rgb::RGB;
    use serde::{
        de::{Error, Unexpected},
        Deserialize, Deserializer, Serializer,
    };

    pub fn serialize<S>(color: &RGB<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:02x}{:02x}{:02x}", color.r, color.g, color.b);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<RGB<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut buf = [0u8; 3];
        let s = <&str>::deserialize(deserializer)?;
        hex::decode_to_slice(s, &mut buf)
            .map_err(|_| Error::invalid_value(Unexpected::Str(s), &"a hexadecimal RGB color"))?;

        Ok(RGB {
            r: buf[0],
            g: buf[1],
            b: buf[2],
        })
    }
}
