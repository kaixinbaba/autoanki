//! Handle with HTTP
//! - Handle the request between CLIENT and server.
//! - Parse the html in response and convert them to specified format
//! - Be the **FACADE** layer for other mods in this project, human's API
use std::collections::HashMap;
use std::fmt::format;

use anyhow::{anyhow, bail, Context, Result};
use cookie::Cookie;
use nipper::Document;
use once_cell::sync::Lazy;
use reqwest::{Client, Error};
use reqwest::header;
use serde_json::Value::Number;

use crate::http::PartOfSpeech::{Adjectives, Adverb, Conjunction, Nouns, Numerals, Preposition, Verb};

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());


/// The parse result from Response.
///
///
///
#[derive(Debug, Default, Clone)]
pub(crate) struct LongMan {
    /// The origin query word
    word: String,

    /// multiple details for the word
    details: Vec<WordDetail>,

}


#[derive(Debug, Default, Clone)]
pub(crate) struct WordDetail {
    /// The IPA sign
    phonetic_symbol: String,

    part_of_speech: PartOfSpeech,

    explanations: Vec<ExplanationDetail>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ExplanationDetail {
    explanation: String,
    sentences: Vec<String>,
    phrases: Vec<PhraseDetail>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct PhraseDetail {
    phrase: String,
    sentences: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub(crate) enum PartOfSpeech {
    Nouns,

    Pronouns,

    #[default]
    Adjectives,

    Numerals,

    Verb,

    Adverb,

    Preposition,

    Conjunction,

    Interjection,

    Article,

}

impl TryFrom<String> for PartOfSpeech {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.trim() {
            "noun" => Ok(Nouns),
            "adjective" => Ok(Adjectives),
            "verb" => Ok(Verb),
            "adverb" => Ok(Adverb),
            "number" => Ok(Numerals),
            "conjunction" => Ok(Conjunction),
            "preposition" => Ok(Preposition),
            _ => bail!("Illegal value '{}'", value)
        }
    }
}

impl From<PartOfSpeech> for String {
    fn from(value: PartOfSpeech) -> Self {
        match value {
            PartOfSpeech::Nouns => "nouns".to_string(),
            PartOfSpeech::Pronouns => "pronouns".to_string(),
            PartOfSpeech::Adjectives => "adjective".to_string(),
            PartOfSpeech::Numerals => "num".to_string(),
            PartOfSpeech::Verb => "verb".to_string(),
            PartOfSpeech::Adverb => "adverb".to_string(),
            PartOfSpeech::Preposition => "preposition".to_string(),
            PartOfSpeech::Conjunction => "conjunction".to_string(),
            PartOfSpeech::Interjection => "interjection".to_string(),
            PartOfSpeech::Article => "article".to_string(),
        }
    }
}


/// Try to save given word in Anki.
///
/// # Examples
///
/// ```
/// save_word("happy".to_string()).await;
/// ```
///
/// # Errors
///
/// Returns an `Err` variant if there's something wrong
///
/// # Arguments
///
/// * `word`: The word that you want to save in Anki
///
/// # Returns
///
/// Returns `Ok(())` if the save operation completes successfully.
pub async fn save_word(word: String) -> Result<()> {
    // Step 1: Access https://www.ldoceonline.com/dictionary/{word} for an explanation of the word
    let url = format!("https://www.ldoceonline.com/dictionary/{}", word.clone());
    let response = CLIENT.get(&url).send().await.context("Failed to get LDOCE response")?;

    // Step 2: Parse the HTML response from the website.
    let lm = parse(word, Document::from(&response.text().await.context("Failed to get LDOCE response body")?));

    // Step 3: Save the result to Anki.
    save(lm).await?;

    // Return Ok(()) to indicate that the function completed successfully.
    Ok(())
}

/// Saves the provided LongMan dictionary item to Anki using the E2E format.
///
/// # Arguments
///
/// * `lm` - A `LongMan` struct representing the dictionary item to save.
///
/// # Errors
///
/// Returns a `Result` containing an error if there was a problem sending the request to Anki or if the
/// server returned an error message.
async fn save(lm: LongMan) -> Result<()> {
    // Convert the LongMan struct to the E2E format.
    let data = convert_e2e(lm);

    // Build the request parameters from the E2E format data.
    let params = build_params(data.clone());

    // Build the cookie for the AnkiWeb session.
    let cookie = build_cookie();

    // Send a POST request to the AnkiWeb server to save the data.
    let resp = CLIENT
        .post("https://ankiuser.net/edit/save")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded; charset=UTF-8")
        .header(header::COOKIE, cookie)
        .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
        .form(&params)
        .send()
        .await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        bail!(resp.text().await?)
    }
}

fn build_params(data: String) -> HashMap<String, String> {
    let mut params = HashMap::new();
    params.insert("nid".to_string(), "".to_string());
    params.insert("data".to_string(), data);
    params.insert("csrf_token".to_string(), "eyJvcCI6ICJlZGl0IiwgImlhdCI6IDE2NzY4OTcwODYsICJ1aWQiOiAiMjMxM2RiYjMifQ.m-3dXu7pTAW-zPL8nf7AAcHmkNTvd35Vt60yUWdVEN0".to_string());
    params.insert("mid".to_string(), "1674395347344".to_string());
    params.insert("deck".to_string(), "1674395027912".to_string());
    params
}

fn build_cookie() -> String {
    let mut cookies = HashMap::new();
    // eyJrIjogImpVZFpSMnFHMW9yZnZpT0wiLCAiYyI6IDIsICJ0IjogMTY3NjQ3MDg5Mn0.IfV3frGBX3d0bz2PVH-AM32xqQekdCcTE-5Y0mykXbc
    cookies.insert("ankiweb", "eyJrIjogImpVZFpSMnFHMW9yZnZpT0wiLCAiYyI6IDIsICJ0IjogMTY3Njg5Njc2MH0.Tb8CPoywrfNK6UQGv4rOwYfD8sxLrgPRZZTT_ofnUwo");
    let mut cookie_jar = cookie::CookieJar::new();
    for (key, value) in cookies {
        let cookie = Cookie::new(key.to_string(), value.to_string());
        cookie_jar.add(cookie);
    }
    cookie_jar.iter().map(|cookie| cookie.to_string()).collect::<Vec<_>>().join("; ")
}

fn convert_e2e(lm: LongMan) -> String {

    let mut inner: Vec<String> = Vec::new();

    inner.push(lm.word.clone());
    inner.push(lm.details.len().to_string());

    for i in 0..=2 {
        if let Some(word_detail) = lm.details.get(i) {
            let ps = word_detail.phonetic_symbol.clone();
            inner.push(word_detail.part_of_speech.clone().into());
            let exp = word_detail.explanations.get(0).unwrap();
            let sentence = exp.sentences.get(0).unwrap();
            inner.push(sentence.clone());
            if let Some(phr) = exp.phrases.get(0) {
                let phr_sentence = if !phr.sentences.is_empty() {
                    phr.sentences.clone().get(0).unwrap().clone()
                } else {
                    "".to_string()
                };
                inner.push(format!("{}<br/>{}", phr.phrase.clone(), phr_sentence));
            } else {
                inner.push("".to_string());
            }

            inner.push(format!("{}<br/>{}", ps, exp.explanation));
        } else {
            for j in 0..=3 {
                inner.push("".to_string());
            }
        }
    }

    format!(r#"[{},""]"#, serde_json::to_string(&inner).unwrap())
}

fn parse(word: String, document: Document) -> LongMan {
    let dictionary = document.select("div.dictionary").first();
    let details: Vec<WordDetail> = dictionary
        .select(".dictentry")
        .iter()
        .filter_map(|content| {
            let phonetic_symbol = content.select(".PronCodes").text().to_string();

            let pos = content.select(".POS").first().text().to_string();

            let part_of_speech = PartOfSpeech::try_from(pos).ok()?;

            let explanations: Vec<ExplanationDetail> = content
                .select(".Sense")
                .iter()
                .enumerate()
                .filter_map(|(index, sense)| {
                    let def = sense.select(".DEF").text().to_string();
                    if def.trim().is_empty() {
                        return None;
                    }
                    let signpost = sense.select(".SIGNPOST").text().to_string();
                    let explanation = if signpost.is_empty() {
                        def
                    } else {
                        format!("[{}] {}", signpost, def)
                    };

                    let sentences: Vec<String> = sense
                        .select(".EXAMPLE")
                        .iter()
                        .map(|exp| {
                            exp.text().to_string().trim().to_owned()
                        }).collect();

                    let phrases: Vec<PhraseDetail> = sense
                        .select(".GramExa, .ColloExa")
                        .iter()
                        .filter_map(|exa| {
                            let phrase = exa.select(".PROPFORMPREP, .PROPFORM").first().text().to_string();
                            if phrase.trim().is_empty() {
                                return None;
                            }
                            let phrase_sentences: Vec<String> = exa.select(".EXAMPLE").iter().map(|exp| {
                                exp.text().to_string().trim().to_owned()
                            }).collect();

                            Some(PhraseDetail {
                                phrase,
                                sentences: phrase_sentences,
                            })
                        })
                        .collect();

                    Some(ExplanationDetail {
                        explanation,
                        sentences,
                        phrases,
                    })
                }).collect();

            Some(WordDetail {
                phonetic_symbol,
                part_of_speech,
                explanations,
            })
        }).collect();

    LongMan {
        word,
        details,
    }
}


#[cfg(test)]
mod test {
    use cookie::Cookie;

    use super::*;

    #[test]
    fn test1() {
        println!("hello world");
    }

    #[tokio::test]
    async fn test_save_anki() {
        /*
            nid
            data
            csrf_token
            mid
            deck
         */
        // let data = r#"[["123","1","123","123","123","123","234","234","5645","345346","435","547","54457","6567"],""]"#.to_string();
        // let data = r#"[["abandon","3","verb","How could she abandon her own child?","to leave someone, especially someone you are responsible for","nouns","They drank and smoked with reckless abandon.","if someone does something with abandon, they behave in a careless or uncontrolled way, without thinking or caring about what they are doing","verb","The company abandoned its takeover bid.","to stop doing or using something because it is too difficult or unsuccessful"],""]"#.to_string();
        let data = r#"[["abb","3","verb","How could she abandon her own child?","to leave someone, especially someone you are responsible for","nouns","They drank and smoked with reckless abandon.","if someone does something with abandon, they behave in a careless or uncontrolled way, without thinking or caring about what they are doing","verb","The company abandoned its takeover bid.","to stop doing or using something because it is too difficult or unsuccessful","547","54457","6567"],""]"#.to_string();
        println!("{:?}", data.clone());
        let params = build_params(data.clone());
        let cookie = build_cookie();

        let resp = CLIENT
            .post("https://ankiuser.net/edit/save")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded; charset=UTF-8")
            .header(header::COOKIE, cookie)
            .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
            .form(&params)
            .send()
            .await.unwrap();

        let x = resp.text().await.unwrap();
        println!("Response: <{:?}>", x);
    }

    #[test]
    fn test_eq() {
        let a = "a".to_string();
        let b = "a".to_string();
        assert_eq!(a, b);
    }

    #[test]
    fn test_json() {
        let v = vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()];
        let json = serde_json::to_string(&v).unwrap();
        println!("{}", json);
    }

    #[test]
    fn test_enum() {
        let x = PartOfSpeech::Adverb;
        let s = String::from(x);
        println!("{}", s);
    }

    #[test]
    fn test_for() {
        for i in 0..=10 {
            println!("{}", i);
        }
    }
}
