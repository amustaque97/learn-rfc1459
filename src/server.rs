use chrono::{DateTime, Local};
use regex::Regex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

// key: addr
// Val: nickname, username
type UserList = Arc<Mutex<HashMap<String, Vec<String>>>>;
// key: addr
// val: connection status — active or inactive
type ServerList = Arc<Mutex<HashMap<String, String>>>;

pub enum Errors {
    ErrNoNickNameGiven = 431,
    ErrNickNameInUse = 433,
    ErrUserDisabled = 446,
    ErrNoSuchServer = 402,
    ErrNeedMoreParams = 461,
    UnknownCommand = -1,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub admin: String,
    pub version: &'static str,
    pub show_users: bool,
    pub motd: Option<&'static str>,
    pub users: UserList,
    pub servers: ServerList,
}

impl Server {
    pub fn new(admin: String) -> Self {
        Server {
            admin,
            version: "1.0",
            motd: None,
            show_users: false,
            users: Arc::new(Mutex::new(HashMap::new())),
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_motd(&mut self, msg: &'static str) {
        self.motd = Some(msg);
    }

    pub fn disable_users(&mut self) {
        self.show_users = true;
    }

    pub fn enable_users(&mut self) {
        self.show_users = false;
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

    // todo: implemnt hostname servername and realname parameter
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

    // todo: add server parameter
    pub async fn users_command(&mut self) -> (Option<Errors>, String) {
        if !self.show_users {
            return (
                Some(Errors::ErrUserDisabled),
                "USERS has been disabled".to_string(),
            );
        }
        let users_list = self.users.lock().unwrap();
        let users: Vec<String> = users_list
            .clone()
            .iter()
            .map(|(_, v)| v.clone().get(1).unwrap().to_string())
            .collect();
        return (None, format!("{}\r\n", users.join("\r\n")));
    }

    // todo add server parameter support
    pub async fn show_version(&self) -> (Option<Errors>, String) {
        (None, format!("{}\r\n", self.version.to_string()))
    }

    // todo add support for <remote server> <server mask>
    pub async fn links_command(&mut self, command_list: Vec<String>) -> (Option<Errors>, String) {
        let servers = self.servers.lock().unwrap();
        let pattern = &command_list[0];
        let regex = String::from(pattern);
        let re: Regex = Regex::new(&regex).unwrap();
        let mut result: Vec<String> = Vec::new();
        for (key, _val) in servers.iter() {
            if re.is_match(key) {
                result.push(key.clone())
            }
        }
        (None, format!("All servers {}\r\n", result.join("\r\n")))
    }

    // todo add server parameter support
    pub async fn show_time(&self) -> (Option<Errors>, String) {
        let now: DateTime<Local> = Local::now();
        (
            None,
            format!("{}\r\n", now.format("%Y-%m-%d %H:%M:%S").to_string()),
        )
    }

    // todo support remote server param
    pub async fn connect_command(&mut self, command_list: Vec<String>) -> (Option<Errors>, String) {
        let servers = self.servers.lock().unwrap();
        if servers.contains_key(&command_list[0]) {
            (
                None,
                format!("Attemp to connect a server to {}", &command_list[0]),
            )
        } else {
            (Some(Errors::ErrNoSuchServer), format!("No such server\r\n"))
        }
    }

    // todo add server parameter support
    pub async fn admin_command(&self) -> (Option<Errors>, String) {
        (None, format!("{}\r\n", self.admin.clone()))
    }

    // todo support operator <o>
    ///! this is identical to the `userhost` command with only a minor diff
    ///! `who` commmand only accepts a single username whereas other command 
    ///! can lookup multiple names. Also `who` command support operator whereas
    ///! `userhost` command can't.
    pub async fn who_command(&self, command_list: Vec<String>) -> (Option<Errors>, String) {
        let users_list = self.users.lock().unwrap();
        let who_user = &command_list[0];
        let users: Vec<String> = users_list
            .clone()
            .iter()
            .filter(|(_k, v)| {
                for val in v.into_iter() {
                    if val == who_user {
                        return true;
                    }
                }
                return false;
            })
            .map(|(k, _v)| k.clone())
            .collect();

        (None, format!("{}\r\n", users.join("\r\n")))
    }

    pub async fn userhost_command(&self, command_list: Vec<String>) -> (Option<Errors>, String) {
        let users_list = self.users.lock().unwrap();
        let users_req: Vec<String> = command_list;
        if users_req.len() == 0 {
            return (Some(Errors::ErrNeedMoreParams), format!("Not enough parameters"));
        }
        let users: Vec<String> = users_list
            .clone()
            .iter()
            .filter(|(_k, v)| {
                for val in v.into_iter() {
                    for u in users_req.iter().cloned() {
                        if &u == val {
                            return true;
                        }
                    }
                }
                return false;
            })
            .map(|(k, _v)| k.clone())
            .collect();
        (None, format!("{}\r\n", users.join("\r\n")))
    }
}

// impl Copy for Server {}
