use crate::proxied_reqwest;
use reqwest::Client;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::*;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Write;
use std::sync::Arc;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EgoName {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FriendsList {
    pub data: Vec<Friend>,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    pub id: u64,
    pub name: String,
}

pub struct Robloxian {
    id: u64,
    name: String,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdNameHash {
    pub names: HashMap<u64, String>,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FriendsListJson {
    pub user_friends: HashMap<u64, Vec<u64>>,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct JointJson {
    names: HashMap<u64, String>,
    user_friends: HashMap<u64, Vec<u64>>,
}

pub trait IoThings: Sized {
    fn load() -> io::Result<Self>;
    fn write(self) -> io::Result<()>;
}

impl IoThings for IdNameHash {
    fn load() -> io::Result<Self> {
        let file = File::open("roblox.json")?;
        let reader = BufReader::new(file);
        let id_name_hash = serde_json::from_reader(reader)?;
        Ok(id_name_hash)
    }
    fn write(self) -> io::Result<()> {
        let json = serde_json::to_string(&self)?;
        let mut file = std::fs::File::create("roblox.json").unwrap();
        file.write_all(json.as_bytes());
        Ok(())
    }
}
impl IoThings for FriendsListJson {
    fn load() -> io::Result<Self> {
        let file = File::open("roblox.json")?;
        let reader = BufReader::new(file);
        let friend_list_json = serde_json::from_reader(reader)?;
        Ok(friend_list_json)
    }
    fn write(self) -> io::Result<()> {
        let json = serde_json::to_string(&self)?;
        let mut file = std::fs::File::create("roblox.json").unwrap();
        file.write_all(json.as_bytes());
        Ok(())
    }
}

impl JointJson {
    pub async fn new(names: HashMap<u64, String>, user_friends: HashMap<u64, Vec<u64>>) -> Self {
        JointJson {
            names,
            user_friends,
        }
    }
    pub fn write(self) -> io::Result<()> {
        let json = serde_json::to_string(&self)?;
        let mut file = std::fs::File::create("roblox.json").unwrap();
        file.write_all(json.as_bytes());
        dbg!("Wrote to file");
        Ok(())
    }
}

impl Robloxian {
    pub async fn create_user(id: u64, hash: &mut IdNameHash) -> Self {
        if let Some(name) = hash.names.get(&id) {
            dbg!("Found user");
            Robloxian {
                id,
                name: name.to_string(),
            }
        } else {
            let get_name = get_name(&id).await;
            hash.names.insert(id, get_name.clone());
            dbg!("Making user");
            Robloxian {
                id,
                name: get_name, // add code to get name from api.
            }
        }
    }
    pub async fn get_friends(
        id: &u64,
        community: &mut HashMap<u64, Vec<u64>>,
        name_map: &mut HashMap<u64, String>,
        my_client: Client,
    ) -> reqwest::Result<()> {
        let url = format!("https://friends.roblox.com/v1/users/{}/friends", id);
        let req = my_client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let friends_list: FriendsList = serde_json::from_str(&req).unwrap();
        let friend_vec: Vec<u64> = friends_list.data.iter().map(|id| id.id).collect();
        friends_list.data.iter().for_each(|friend| {
            name_map.insert(friend.id, friend.name.clone());
        });
        community.insert(*id, friend_vec);
        Ok(())
    }
}

pub async fn get_name(id: &u64) -> String {
    let client = proxied_reqwest::create_client().await.unwrap();
    let url = format!("https://users.roblox.com/v1/users/{}", id);
    let req = client.get(&url).send().await.unwrap().text().await.unwrap();
    let user: EgoName = serde_json::from_str(&req).unwrap();
    user.name
}

// Create the write function to write the IdNameHash to file, also do the same for the
// FriendsListJson.

// Create function to make the friend_list_json from api calls too.
