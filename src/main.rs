use std::{collections::HashMap, env, error::Error, future::Future, pin::Pin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    // println!("args: {:#?}", args);
    let google = Google::new();
    let result = match args.len() {
        3 => {
            let (text, to) = (&args[1], &args[2]);
            let to = Lang::from(to.to_owned());
            let from = match to {
                Lang::EN => Lang::JP,
                Lang::JP => Lang::EN,
            };
            google.translate(text.as_str(), &from, &to).await
        }
        4 => {
            let (text, to, from) = (
                &args[1],
                Lang::from((&args[2]).to_owned()),
                Lang::from((&args[3]).to_owned()),
            );
            google.translate(text.as_str(), &from, &to).await
        }
        other => {
            panic!(format!("argument is invalid. args = {:#?}", args))
        }
    };

    match result {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        }
        Err(err) => {
            println!("{:#?}", err);
            Ok(())
        }
    }
}

type TranslateResult = Result<String, TranslateError>;
trait Translator {
    fn translate(
        &self,
        text: &str,
        from: &Lang,
        to: &Lang,
    ) -> Pin<Box<dyn Future<Output = TranslateResult> + Send>>;
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct TranslateError {
    message: String,
}
impl TranslateError {
    pub fn new(msg: String) -> Self {
        Self { message: msg }
    }
}

enum Lang {
    JP,
    EN,
}
impl From<String> for Lang {
    fn from(s: String) -> Self {
        match s.as_str() {
            "jp" => Lang::JP,
            "en" => Lang::EN,
            unknown => panic!("unknown lang({}) is given", s),
        }
    }
}

struct Google {
    project_id: String,
    access_token: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize)]
struct GoogleRequestBody {
    contents: Vec<String>,
    source_language_code: String,
    target_language_code: String,
}
impl GoogleRequestBody {
    pub fn new(text: &str, from: &Lang, to: &Lang) -> Self {
        Self {
            contents: vec![text.to_string()],
            source_language_code: Self::language_text(from),
            target_language_code: Self::language_text(to),
        }
    }
    fn language_text(lang: &Lang) -> String {
        match lang {
            Lang::JP => "ja".to_string(),
            Lang::EN => "en".to_string(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize)]
struct GoogleResponseBody {
    translations: Vec<GoogleResponseTranslation>,
}
impl GoogleResponseBody {
    pub fn text(&self) -> String {
        self.translations.get(0).unwrap().translated_text.to_owned()
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize)]
struct GoogleResponseTranslation {
    translated_text: String,
}

impl Google {
    pub fn new() -> Self {
        let project_id = std::env::var("GOOGLE_PROJECT_ID").expect("must set GOOGLE_PROJECT_ID");
        let access_token = std::env::var("GOOGLE_ACCESS_TOKEN")
            .expect("must set GOOGLE_ACCESS_TOKEN using `gcloud auth application-default print-access-token`");
        Self {
            project_id,
            access_token,
        }
    }

    fn translate_text_url(&self) -> String {
        format!(
            "https://translate.googleapis.com/v3beta1/projects/{}:translateText",
            self.project_id
        )
    }
}

impl Translator for Google {
    // async fn translate(&self, text: &str, from: &Lang, to: &Lang) -> TranslateResult {
    fn translate(
        &self,
        text: &str,
        from: &Lang,
        to: &Lang,
    ) -> Pin<Box<dyn Future<Output = TranslateResult> + Send>> {
        let body = GoogleRequestBody::new(text, from, to);

        let client = reqwest::Client::new();
        let req = client
            .post(self.translate_text_url())
            .header("Authorization", format!("Bearer {}", &self.access_token))
            .json(&body)
            .send();
        let res = async move {
            req.await
                .unwrap()
                .json()
                .await
                .map(|res: GoogleResponseBody| res.text())
                .map_err(|err| TranslateError::new(format!("error! {:#?}", err)))
        };
        Box::pin(res)
    }
}
