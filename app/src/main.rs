use serde::{Serialize, Deserialize};
use std::fs;
use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

static OPENAI_CHAT_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Serialize, Deserialize, Debug)]
struct Secrets {
    openai_key: String,
}

impl Secrets {
    fn new() -> Self {
        let secrets: Result<String, _> = fs::read_to_string("secrets.json");
        if secrets.is_err() {
            panic!("Could not read secrets.json. Please create it and add your OpenAI key and other information.");
        }

        let secrets: Result<Secrets, _> = serde_json::from_str(&secrets.unwrap());
        if secrets.is_err() {
            panic!("Could not parse secrets.json. Please make sure it is valid JSON.");
        }

        secrets.unwrap()
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChatTurn {
    role: String,
    content: String
}

type ChatLog = Vec<ChatTurn>;


#[derive(Serialize, Deserialize, Clone, Debug)]
struct OpenAIChatRequest {
    model: String,
    messages: ChatLog,
    temperature: f32,
    top_p: f32,
    n: u32,
    stream: bool,
    stop: Option<Vec<String>>,
    max_tokens: u32,
    presence_penalty: f32,
    frequency_penalty: f32,
    logit_bias: HashMap<String, f32>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OpenAIChatResponseChoice {
    index: u32,
    logprobs: Option<HashMap<String, f32>>,
    finish_reason: String,
    message: ChatTurn
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OpenAIChatResponseUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    choices: Vec<OpenAIChatResponseChoice>,
    usage: OpenAIChatResponseUsage
}

impl OpenAIChatRequest {
    fn new() -> Self {
        OpenAIChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![],
            temperature: 1.0,
            top_p: 1.0,
            n: 1,
            stream: false,
            stop: None,
            max_tokens: 512,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            logit_bias: HashMap::new()
        }
    }

    fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(ChatTurn {
            role: role.to_string(),
            content: content.to_string()
        });
    }

    async fn post_request(&self, secrets: &Secrets, reqwest: &mut reqwest::Client) -> Option<OpenAIChatResponse> {
        let auth = String::from("Bearer ") + &secrets.openai_key;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth).unwrap());
        let body = serde_json::to_string(&self).unwrap();
        // println!("body: {:?}", body);
        let request = reqwest.post(OPENAI_CHAT_ENDPOINT)
            .headers(headers)
            .body(body)
            .build()
            .unwrap();
        // println!("{:?}", request);

        let response = reqwest.execute(request).await;

        if let Ok(response) = response {
            // println!("Response: {:?}", response);
            
            if response.status() != 200 {
                println!("Error body: {:?}", response.text().await);
            } else {
                let response = response.json::<OpenAIChatResponse>().await;
                if let Ok(response) = response {
                    return Some(response);
                } else {
                    println!("Error: {:?}", response);
                    return None
                }
            }
            return None;
        } else {
            println!("Error: {:?}", response);
            return None
        }
    }

}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let secrets: Secrets = Secrets::new();
    let mut reqwest: reqwest::Client = reqwest::Client::new();

    let mut chat = OpenAIChatRequest::new();
    chat.temperature = 0.2;
    chat.add_message("system", "You are an AI helping the user to convert transcribed speech into Rust source code.");
    chat.add_message("user", "Below, I will give you an utterance transcribed directly from speech. Attempt to render this speech in idiomatic Rust such that I could copy-paste it into an existing program. It may not be a full line of code, and you may need to format variables using TitleCase or snake_case. Make the best guess you can. Do not output anything but the Rust code. If you need to ask a question to clarify, prepend ??? to your reply.");
    chat.add_message("assistant", "Okay, I'm ready!");
    chat.add_message("user", "ranking dot add item of and then construct an item literal with name equal to \"A 747 jet\", that's a lowercase string, and positive equal to false");
    chat.add_message("assistant", "ranking.add_item(Item{\n    name: String::from(\"a firm kick in the groin\",\n    positive: false,\n});");
    chat.add_message("user", "let fit equal ranking dot players dot get mute of 1, in quotes, dot unwrap dot fit of ranking dot items dot length");
    chat.add_message("assistant", "let fit = ranking.players.get_mut(\"1\")\n    .unwrap()\n    .fit(ranking.items.len());");
    
    //chat.add_message("user", "let response equal chat dot post request of borrow secrets, mutable borrow request, after the parens dot await");
    // chat.add_message("user", "pound CFG target OS as M script in");
    // chat.add_message("assistant", "??? I'm not sure what you mean by \"M script\".");
    // chat.add_message("user", "public fun default host returns a host. return web audio host colon colon new dot expect of the string \"the default host should always be available\", then dot into and call it.");
    // chat.add_message("user", "add a pound CFG which rules out linux, dragonfly, free bsd, mac os, ios, android, and M script in.");
    chat.add_message("user", "make a public struct device deriving clone partial equals and equals and it has two members, one of type audio device id and an is default of type boolean. make the audio device id pub crate.");
    let response = chat.post_request(&secrets, &mut reqwest).await;
    
    println!("{:?}", response);

    println!("{}", &response.unwrap().choices[0].message.content);
    
    Ok(())
}