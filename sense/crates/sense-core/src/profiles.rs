//! `app-profiles`: the only place where knowledge about specific applications lives.

use sense_schema::Category;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct AppProfile {
    pub canonical: String,
    #[serde(default)]
    pub category: Category,
    #[serde(rename = "match")]
    pub matches: Vec<String>,
    #[serde(default)]
    pub copy_shortcut: Option<String>,
    #[serde(default)]
    pub paste_shortcut: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Profiles {
    #[serde(default)]
    app: Vec<AppProfile>,
}

impl Profiles {
    pub fn builtin() -> Self {
        toml::from_str(include_str!("../data/app-profiles.toml")).expect("built-in app-profiles parse")
    }

    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Look an application up by the raw id its OS reports. Case-sensitive first, then not.
    pub fn lookup(&self, raw_id: &str) -> Option<&AppProfile> {
        self.app
            .iter()
            .find(|p| p.matches.iter().any(|m| m == raw_id))
            .or_else(|| {
                self.app
                    .iter()
                    .find(|p| p.matches.iter().any(|m| m.eq_ignore_ascii_case(raw_id)))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_canonical_name_for_three_operating_systems() {
        let p = Profiles::builtin();
        for raw in ["org.mozilla.firefox", "firefox.exe", "Navigator.firefox"] {
            assert_eq!(p.lookup(raw).map(|a| a.canonical.as_str()), Some("firefox"), "{raw}");
        }
        assert_eq!(
            p.lookup("org.keepassxc.KeePassXC").unwrap().category,
            Category::PasswordManager
        );
        assert!(p.lookup("com.example.unknown").is_none());
    }

    #[test]
    fn terminals_never_get_plain_ctrl_c() {
        let p = Profiles::builtin();
        assert_eq!(
            p.lookup("org.gnome.Ptyxis").unwrap().copy_shortcut.as_deref(),
            Some("ctrl+shift+c")
        );
    }
}
