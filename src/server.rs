use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

// key: addr
// Val: nickname, username
type UserList = Arc<Mutex<HashMap<String, Vec<String>>>>;

pub enum Errors {
    ErrNoNickNameGiven = 431,
    ErrNickNameInUse = 433,
    UnknownCommand = -1,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub motd: Option<&'static str>,

    pub users: UserList,
}

impl Server {
    pub fn new() -> Self {
        Server {
            motd: None,
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_motd(&mut self, msg: &'static str) {
        self.motd = Some(msg);
    }

    pub async fn nick_command(
        &mut self,
        command_list: Vec<String>,
        addr: SocketAddr,
    ) -> (Option<Errors>, String) {
        let empty = "".to_string();
        let nickname = command_list
            .get(0)
            .unwrap_or_else(|| &empty)
            .to_string()
            .clone()
            .to_lowercase();
        println!("Requested nickname {:?}", nickname);

        if nickname.len() < 1 {
            return (
                Some(Errors::ErrNoNickNameGiven),
                "No Nickname given\r\n".to_string(),
            );
        }

        let mut user_list = self.users.lock().unwrap();
        if user_list.contains_key(&nickname) {
            // todo kill command and kick out all the existing nickname
            return (
                Some(Errors::ErrNickNameInUse),
                format!("{} Nickname is already in use\r\n", nickname),
            );
        } else {
            let mut val = Vec::new();
            val.push(nickname.clone());
            user_list.insert(addr.to_string(), val);
            return (None, format!("Introducing new nick \"{}\" \r\n", nickname));
        }
    }

    // todo: implemnt hostname servername and realname command
    pub async fn user_command(
        &mut self,
        command_list: Vec<String>,
        addr: SocketAddr,
    ) -> (Option<Errors>, String) {
        let username = command_list.get(0).unwrap().clone().to_lowercase();
        println!("Requested username {:?}", username);
        let mut user_list = self.users.lock().unwrap();

        let val: &mut Vec<String> = user_list.get_mut(&addr.to_string()).unwrap();
        val.push(username);
        return (None, "Username registered successfully\r\n".to_string());
    }

    pub async fn users_command(&mut self) -> (Option<Errors>, String) {
        let users_list = self.users.lock().unwrap();
        let users: Vec<String> = users_list
            .clone()
            .iter()
            .map(|(_, v)| v.clone().get(1).unwrap().to_string())
            .collect();
        return (None, format!("{}\r\n", users.join("\r\n")));
    }
}

// impl Copy for Server {}
