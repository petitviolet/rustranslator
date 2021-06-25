use std::{collections::HashMap, env, error::Error, future::Future, pin::Pin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    println!("args: {:#?}", args);
    let result = match args.len() {
        3 => {
            let (text, to) = (&args[1], &args[2]);
            let to = Lang::from(to.to_owned());
            let from = match to {
                Lang::EN => Lang::JP,
                Lang::JP => Lang::EN,
            };
            Google {}.translate(text.as_str(), &from, &to).await
        }
        4 => {
            let (text, to, from) = (
                &args[1],
                Lang::from((&args[2]).to_owned()),
                Lang::from((&args[3]).to_owned()),
            );
            Google {}.translate(text.as_str(), &from, &to).await
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
// trait Translator {
//   fn translate(text: &str, from: &Lang, to: &Lang) -> Pin<Box<TranslateResult + Send>>;
// }

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

struct Google;
impl Google {
    const GOOGLE_BASE_URL: &'static str =
        "https://translation.googleapis.com/language/translate/v2";

    pub async fn translate(&self, text: &str, from: &Lang, to: &Lang) -> TranslateResult {
        let mut body = HashMap::new();
        body.insert("q", text);
        body.insert("source", self.language_text(from));
        body.insert("target", self.language_text(to));

        let client = reqwest::Client::new();
        let res = client
            .post(Self::GOOGLE_BASE_URL)
            .json(&body)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .map_err(|err| TranslateError::new(format!("error! {:#?}", err)));

        res
    }

    fn language_text(&self, lang: &Lang) -> &str {
        match lang {
            Lang::JP => "ja",
            Lang::EN => "en",
        }
    }
}
